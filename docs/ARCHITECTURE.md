# Arquitetura — 0.4.0

## Motor gráfico compartilhado

`glass/optics.mjs` contém SDF, perfil de lente convexa, lookup e rasterização simétrica. `maps.ts` mantém LRU de oito mapas por WebView, indexado por dimensão, raio, espessura, borda e qualidade. Intensidade de refração, cor, iluminação e dispersão não invalidam o mapa. Não há compartilhamento de heap entre janelas WebView2; esse limite é explícito.

`LiquidGlass.tsx` usa IDs de filtro exclusivos e separação RGB opcional sobre uma única textura. Não monta mapa nem canvas no modo animado. A textura estática redesenha por eventos coalescidos em requestAnimationFrame. Eventos de movimento, DPI, resize, retorno à visibilidade e preferências substituem o polling anterior de 33/1.000 ms. Eventos recebidos durante desenho pendente geram um único redesenho posterior. Desmontagem remove listeners, observadores e callbacks.

`wallpaper::read_cached` serializa leitura COM/arquivo sob mutex e guarda o resultado por 15 segundos para todos os widgets. `materials.ts` define seis presets. `spring.mjs` preserva posição e velocidade ao retargeting, e `motion.ts` registra ticker GSAP somente enquanto a superfície se move. O painel usa o mesmo renderer na prévia ilustrativa; não representa pixels do desktop.

Settings ganhou campos aditivos com defaults: material custom, espessura 18, borda 26, dispersão 0,1, difusão 0,35, qualidade balanced e movimento ligado. Schema 2 permanece compatível; valores existentes não são resetados. Sincronização inclui os novos campos e mantém conteúdo/geometria individuais. Backend valida enums, finitude e limites.

O comportamento nativo DesktopWidget não foi substituído. Os papéis FloatingIsland, DesktopBar, Dock e Flyout ainda serão implementados separadamente conforme ROADMAP; a lista de IDs continua restrita aos cinco widgets.

## Aplicativo em segundo plano

A bandeja é criada antes de mostrar o painel. O painel inicia invisível; lançamento normal o mostra, `--background` apenas restaura os widgets. Fechamento é interceptado e minimização oculta o painel. O menu nativo oferece abrir e sair explicitamente. O plugin single-instance encaminha a segunda abertura para o painel existente.

`startup.rs` registra/remove somente o valor Liquid Desktop na chave Run de HKCU, com caminho do executável entre aspas e argumento `--background`. A consulta lê o Registro, sem duplicar a preferência no JSON. Nenhuma configuração é ativada automaticamente durante instalação/build. Restrições externas do Gerenciador de Tarefas continuam sob controle do usuário.

`Layout.sync_theme` tem default false para manter compatibilidade com 0.2.0. Sincronização copia apenas campos de material para todos os tipos. A operação persiste uma única versão do layout sob mutex, depois publica as configurações para cada janela. Conteúdo, geometria e estado ativo permanecem independentes.

## Janelas e interação

`native.rs` mantém os mecanismos validados: HWND top-level, TOOLWINDOW sem always-on-top, região arredondada, subclass WM_NCCALCSIZE/WM_NCPAINT instalada na thread proprietária, movimento em coordenadas físicas. O painel conserva a moldura normal. `maintain_desktop_layers` reposiciona o conjunto de widgets acima de Progman/WorkerW, sem mudar janelas externas; a ordem entre widgets é estável.

`main.rs` é a fronteira de comandos. Os IDs permitidos são clock, analog, calendar, notes e system. Criar/configurar/remover exige a janela main; salvar notas exige notes; consultar hardware exige system. O arraste usa o label da janela chamadora e não pode movimentar outro widget. A criação é iniciada fora da thread UI, e configuração nativa/subclass é encaminhada por run_on_main_thread.

## Preferências e ciclo de vida

`settings.rs` contém Settings por widget e Layout schema 2. O carregamento migra o JSON antigo para clock, preenche tipos ausentes com defaults e valida limites. Erros de arquivo bloqueiam sobrescrita automática. A atualização do painel envia patches de campos, evitando substituir notas por uma cópia antiga das configurações. enabled é alterado somente por criar/remover. Notas usam comando dedicado.

Persistência cria backup inicial, grava arquivo temporário e renomeia. Acesso ao layout é serializado por mutex. Os eventos settings são enviados às janelas afetadas; layout é enviado ao painel. Os widgets ativos são restaurados sequencialmente após iniciar o event loop. Fechar o painel o oculta na bandeja; Sair encerra o processo e todas as janelas.

## Widgets e material

`src/widgets/catalog.tsx` descreve a coleção. `WidgetWindow.tsx` é o shell comum: preferências, arraste, vidro e erros. Componentes de relógio, calendário, notas e sistema são independentes. `useWidgetDrag` e `dragSession.mjs` usam captura de ponteiro e IPC serializado. Controles interativos são excluídos da área de arraste.

`LiquidGlass.tsx` calcula o mapa de deslocamento conforme largura e altura. Apenas static_wallpaper=true monta a textura e consulta o wallpaper via COM. No modo animado, nenhuma imagem estática é carregada. A cor, borda e luz continuam disponíveis.

## Provedor do sistema

`monitor.rs` cria System e Networks de sysinfo sob demanda. Não atualiza a lista de processos. Mantém baseline de CPU, cache mínimo de 1 segundo e deltas de rede divididos pelo tempo real decorrido. O frontend pede amostras a cada 2 segundos, suspende consultas quando oculto e mantém no máximo 30 amostras reais. Nenhum gráfico é pré-preenchido. Fechar/remover o widget descarta o provedor.

## Painel

`src/panel/Panel.tsx`: coleção de cinco tipos e inspector do selecionado. As gravações são serializadas, os comandos alteram somente os campos enviados e os eventos refletem posição atual após arraste. Tamanho/posição numéricos são aplicados ao sair do campo ou pressionar Enter. O painel não representa uma simulação do desktop.

## Próximos passos

Validar visualmente os novos widgets em múltiplos DPI, medir custo de cinco WebViews, ampliar cache de wallpaper compartilhado, adicionar múltiplas instâncias e novos provedores reais. Clima, mídia, temperatura/GPU, galeria, dock e Dynamic Island permanecem etapas futuras.
