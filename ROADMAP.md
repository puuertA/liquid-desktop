# Liquid Desktop — evolução incremental

Este roteiro acompanha o escopo Next Generation. Uma fase não é considerada validada só porque compila. Não há dados de mídia, notificações, temperatura ou conectividade simulados.

## 1. Material gráfico — 0.4.0, primeiro incremento

Implementado: lente convexa com centro neutro, lookup óptico, dispersão RGB, espessura e faixa de borda, seis materiais, qualidade selecionável, contraste Adaptive sobre textura estática, reflexos de cursor, resposta de mola com GSAP, cache limitado de mapas por WebView, cache de wallpaper compartilhado no Rust e redesenho por eventos. Estúdio de materiais no painel. Migração sem substituir valores anteriores.

Validação matemática, migração, compilação e regressões nativas registradas em docs/VALIDATION.md. Conferência visual no WebView2 e comparação do consumo total com cinco janelas ainda pendentes: ferramenta de UI nativa indisponível neste ambiente. Isso impede declarar a fase visualmente aprovada.

Fusão de superfícies e morphing de geometria ficam associados ao consumidor real da fase 2; não serão apresentados como uma ilha fictícia nesta entrega. O motor mantém os widgets existentes utilizáveis.

## 2. Dynamic Island — próxima prioridade, ainda não implementada

- Papel FloatingIsland independente de DesktopWidget. Monitor selecionável; floating/notch; hit region acompanha a geometria; esconder em fullscreen.
- Máquina de estados Idle, Compact, Expanded, Notification e Multi-Activity, com prioridades, expiração e restauração. Testes de interrupção/reversão antes de integrar provedores.
- GSMTC: sessão selecionável, metadados/capa, comandos realmente suportados e progresso real. Core Audio para volume. Temporizador com relógio monotônico; cancelamento explícito.
- Morphing de largura/altura/raio, mola interrompível, capa compartilhada, fusão entre atividades e conteúdo legível. Sem overlay com janela retangular invisível interceptando o desktop.
- Gate: comprovar mídia e volume reais, clique fora da forma, resolução/DPI, fullscreen e recuperação do Explorer. Não pressupor que o mecanismo de widgets serve para overlays.

## 3. Top Bar e Control Center

DesktopBar por monitor, 28–42 unidades lógicas, módulos reordenáveis, transparência e ocultação. Integrar ilha sem duplicar seus serviços. Flyout ancorado ao controle de origem. Usar informações reais de áudio, energia, conectividade e processo ativo; APIs indisponíveis mostram estado/atalho legítimo. AppBar para reserva opcional de espaço, desligada por padrão.

## 4. Dock

Enumeração de aplicativos/janelas e ícones reais, fixação, abertura/restauração/minimização, menus e ordenação. Magnificação por distância, janela com região interativa coerente, coexistência com a barra original por padrão. Investigar Seelen UI/Windhawk sem injeção no Explorer. Ocultar barra original exige opção explícita e recuperação; substituição experimental só após estabilizar coexistência.

## 5. Ecossistema

Launcher com atalho, teclado, aplicativos e busca de arquivos somente em locais autorizados. Reformular widgets e painel com navegação por superfícies; importar/exportar layouts versionados. Novos widgets reais: Now Playing, clima com provedor, galeria autorizada, bateria, armazenamento, timer e atalhos. Clipboard e notificações de terceiros são investigações separadas e opcionais, não permissões implícitas.

## 6. Validação contínua e distribuição

Repetir testes relevantes em cada fase. Benchmark CPU/GPU/RAM do processo e descendentes WebView2 em repouso, arraste, mídia e animação. Matriz Windows 10/11, Win+D, Alt+Tab, fullscreen, DPI misto, mudanças de monitor, bloqueio/desbloqueio e Explorer reiniciado. Installer e documentação devem distinguir resultados medidos, inspeção visual e pendências. A meta de 60 FPS ainda não está comprovada.
