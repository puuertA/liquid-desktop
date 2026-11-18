# Arquitetura — 0.6.0

## Top Bar

`topbar.rs` implementa uma TOOLWINDOW transparente e sem moldura independente dos widgets. Configuração serde com defaults preserva layouts schema 2, iniciando desativada. Um mutex serializa criação/restauração/alteração; falhas restauram a configuração anterior. A manutenção em UI usa coordenadas físicas do monitor, cache de geometria e a mesma detecção de fullscreen da ilha.

O HWND tem altura de 36 unidades lógicas e margem de 8. `bar_region` une dois retângulos arredondados quando ilha/barra compartilham o monitor efetivo, retirando 480 unidades centrais da região de hit testing. Sem ilha, a região volta a uma cápsula única. A forma é atualizada quando configuração, topologia ou coordenadas mudam, não em cada amostra de métricas. Nenhum Acrylic ou blur de compositor é aplicado.

`topbar_snapshot` lê a configuração, compartilha `State.monitor` com o widget Sistema e usa Core Audio no worker. Polling de 2 segundos tem somente uma chamada em andamento e pausa com document.hidden. `topbar_volume` e `topbar_open_panel` aceitam apenas a janela topbar; `update_topbar` aceita somente main. O volume é enviado ao soltar o slider ou concluir uma tecla, preservando o valor em edição contra atualizações periódicas. Não há capacidades adicionais de shell ou acesso a arquivos.

Frontend usa containers responsivos por segmento. Módulos perdem detalhes em espaço reduzido; não há dados fictícios para preencher valores ausentes. A barra não registra AppBar nem altera área útil de monitores. Control Center e múltiplas instâncias permanecem pendentes.

A 0.5.2 remove o comando de Acrylic, sua dependência direta e o controle de desfoque. O compositor pintava além de SetWindowRgn no desktop do usuário. O campo antigo native_blur é ignorado por serde e desaparece ao salvar; a janela nova não aplica efeito nativo. Preservadas animação, recorte e iluminação CSS. A correção exige encerrar o processo anterior e iniciar o atualizado.

## FloatingIsland

`island.rs` gerencia configuração, ciclo de vida, monitor, temporizador e comandos autorizados apenas para a janela island (configuração somente main). Layout schema 2 ganhou campo island com default desativado; a migração preserva widgets/notas. Um mutex de ciclo de vida serializa ativação e restauração. A ilha é uma janela TOOLWINDOW de 480×340 unidades lógicas, independente da lista IDS e da ordenação DesktopWidget.

SetWindowRgn recebe somente o contorno visível, centralizado na janela; margens transparentes não pertencem à região nativa. Notch combina retângulo superior e contorno arredondado. Frontend mantém no máximo uma atualização de região em voo e coalesce frames intermediários, enviando a geometria final. Mudanças de DPI reenviam o contorno. A geometria visual usa ticker GSAP somente durante transições.

`media.rs` inicializa WinRT/COM no worker, mantém manager e identidades de sessões GSMTC, limita a lista a 16 sessões e as capas a 2 MiB. Não aceita IDs arbitrários de aplicativos para controle: ações resolvem uma sessão enumerada. Progresso deriva da posição, timestamp e taxa reais do provedor. Core Audio usa IMMDeviceEnumerator/IAudioEndpointVolume para saída multimídia padrão. Não há captura/loopback de áudio ou tela. Leituras usam polling de 2 s somente com ilha visível e uma solicitação em voo; isso ainda deve evoluir para eventos compartilhados.

Temporizador é um Instant no runtime Rust, não depende do intervalo do frontend e não é persistido entre execuções. Notificações e arbitragem são módulos puros testáveis; timer concluído tem prioridade 3, volume/início têm prioridade 1, duração de 4,5 s. Sair/desativar cancela o timer. A manutenção de 1 s verifica fullscreen e posição do monitor; jogos exclusivos e mudanças de escala ainda dependem de validação interativa.

## Motor gráfico compartilhado

`glass/optics.mjs` contém SDF, perfil de lente convexa, lookup e rasterização simétrica. `maps.ts` mantém LRU de oito mapas por WebView, indexado por dimensão, raio, espessura, borda e qualidade. Intensidade de refração, cor, iluminação e dispersão não invalidam o mapa. Não há compartilhamento de heap entre janelas WebView2; esse limite é explícito.

`LiquidGlass.tsx` usa IDs de filtro exclusivos e separação RGB opcional sobre uma única textura. Não monta mapa nem canvas no modo animado. A textura estática redesenha por eventos coalescidos em requestAnimationFrame. Eventos de movimento, DPI, resize, retorno à visibilidade e preferências substituem o polling anterior de 33/1.000 ms. Eventos recebidos durante desenho pendente geram um único redesenho posterior. Desmontagem remove listeners, observadores e callbacks.

`wallpaper::read_cached` serializa leitura COM/arquivo sob mutex e guarda o resultado por 15 segundos para todos os widgets. `materials.ts` define seis presets. `spring.mjs` preserva posição e velocidade ao retargeting, e `motion.ts` registra ticker GSAP somente enquanto a superfície se move. O painel usa o mesmo renderer na prévia ilustrativa; não representa pixels do desktop.

Settings ganhou campos aditivos com defaults: material custom, espessura 18, borda 26, dispersão 0,1, difusão 0,35, qualidade balanced e movimento ligado. Schema 2 permanece compatível; valores existentes não são resetados. Sincronização inclui os novos campos e mantém conteúdo/geometria individuais. Backend valida enums, finitude e limites.

O comportamento nativo DesktopWidget não foi substituído. FloatingIsland tem uma janela independente desde 0.5.0; Top Bar desde 0.6.0. Dock e Flyout continuam previstos no ROADMAP. A lista de IDs de widgets continua restrita aos cinco widgets.

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

Validar visualmente widgets e Dynamic Island em múltiplos DPI, medir custo das WebViews, ampliar cache de wallpaper compartilhado, adicionar múltiplas instâncias e novos provedores reais. Ampliar a validação dos comandos de mídia nos players. Clima, temperatura/GPU, galeria, Control Center e Dock permanecem etapas futuras.
