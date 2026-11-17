# Liquid Desktop 0.4.0

Primeiro incremento do plano [Next Generation](ROADMAP.md): motor óptico e materiais. Os cinco widgets, bandeja, temas sincronizados e inicialização opcional continuam disponíveis. Dynamic Island, Top Bar, Control Center e Dock ainda não estão implementados nesta versão.

## Motor óptico e materiais

No painel, selecione um widget e escolha **Clear**, **Thin**, **Regular**, **Frosted**, **Dark** ou **Adaptive**. A prévia é um estúdio com textura ilustrativa, processada pelo mesmo renderer usado no desktop. Cor e material continuam podendo ser sincronizados entre widgets.

Novos ajustes: espessura óptica, largura da borda convexa, dispersão cromática, difusão da textura e qualidade. O centro da lente permanece neutro e a refração se concentra na borda. O material Adaptive mede a luminosidade da textura estática local para aumentar o contraste; com Wallpaper Engine utiliza o contraste manual. Nenhuma imagem de outras janelas ou vídeo do Wallpaper Engine é capturada.

Modo Econômico reduz a resolução do mapa e usa uma passagem sem dispersão. Equilibrado usa mapas até 384 px; Alta, até 640 px. A resposta elástica usa uma mola interrompível e respeita movimento reduzido. [Detalhes de movimento](docs/MOTION.md). Preferências 0.3.0 são carregadas como material Personalizado, sem substituir cores, posições, notas ou valores anteriores.

A conferência visual no WebView2 permanece pendente por indisponibilidade da automação de interface neste ambiente. Testes e benchmark isolado constam em [validação](docs/VALIDATION.md); não representam uma medição de FPS ou de consumo total do aplicativo.

## Bandeja, temas e inicialização

Minimizar ou fechar o painel mantém os widgets ativos na bandeja do sistema. Clique no ícone para reabrir; clique com o botão direito e escolha **Sair do Liquid Desktop** para encerrar. Uma segunda abertura do app reaproveita a instância existente.

Em **Material**, ative **Sincronizar tema de todos os widgets** para aplicar o material selecionado à coleção inteira, inclusive widgets ocultos. Cor, intensidade, reflexos, contraste, refração e modo de wallpaper são compartilhados; posições, tamanhos, notas e opções dos relógios continuam individuais. Desativar a sincronização permite editar cada um novamente. Temas prontos: Cristal, Grafite, Esmeralda e Âmbar.

**Iniciar com o Windows** registra o executável instalado apenas para o usuário atual em HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run. Ao entrar no Windows, restaura os widgets sem abrir o painel. Ative essa opção na versão instalada, não em uma compilação de desenvolvimento. A opção começa desativada e não é ligada pelo instalador. Se bloquear a inicialização no Gerenciador de Tarefas, reative-a também lá. Desative a opção antes de desinstalar para remover a entrada de inicialização.

Widgets nativos para o desktop Windows, com Tauri 2, Rust, React/TypeScript e vidro compartilhado. Cada widget é uma janela transparente independente, sem moldura, posicionada acima do desktop e abaixo de aplicativos. Compatível com o modo translúcido usado sobre Wallpaper Engine; não é uma página simulando um desktop.

## Usar

Encerre a versão antiga pelo menu **Sair do Liquid Desktop** na bandeja antes de instalar a atualização. O instalador fica em `src-tauri/target/release/bundle/nsis/Liquid Desktop_0.4.0_x64-setup.exe`.

No painel, escolha um widget na coleção e clique **Adicionar**. Selecione o card para configurar apenas aquele widget. **Remover** fecha sua janela e mantém suas preferências. Widgets ativos são reabertos ao iniciar o app.

| Widget | Funcionalidade |
|---|---|
| Relógio digital | Hora e data locais; segundos opcionais; formato 12/24 horas |
| Relógio analógico | Ponteiros reais; ponteiro de segundos opcional |
| Calendário | Mês atual, navegação entre meses e botão Hoje; não acessa agendas |
| Notas rápidas | Texto simples editável no desktop, salvo localmente; até 8.000 caracteres |
| Monitor do sistema | CPU, RAM, download e upload reais; gráficos das últimas 30 amostras |

Cada widget tem posição X/Y em pixels físicos, largura e altura em unidades lógicas, cor, intensidade da cor, reflexos, contraste, refração estática e modo de edição próprios. O tamanho pode ser alterado pelo painel (200–640 × 180–600); **Usar tamanho sugerido** restaura uma composição adequada ao tipo.

Ative **Permitir arrastar**. Arraste pelo fundo ou título do widget. Esc cancela; soltar salva a posição. Botões do calendário e a área de texto das notas continuam interativos e não iniciam arraste. A moldura nativa é removida via tratamento Win32 na thread da janela, preservando a correção confirmada na versão 0.1.4.

## Wallpaper Engine e vidro

**Fundo animado / Wallpaper Engine** vem ligado por padrão. Usa transparência, cor, bordas e reflexos, sem copiar a imagem estática antiga para dentro do widget. Não há refração do vídeo nem captura contínua de tela.

Para um wallpaper estático do Windows, desmarque a opção. `IDesktopWallpaper` fornece a imagem real por monitor, que é mapeada e refratada por SVG. O mapa acompanha o tamanho do widget. Consulte mudanças a cada 15 segundos somente nesse modo. Para wallpapers claros, aumente **Contraste de leitura**.

## Dados reais e limites

O monitor usa `sysinfo` 0.37.2 com apenas os recursos de sistema/rede. Atualiza a cada 2 segundos enquanto o documento está visível. CPU precisa de uma amostra inicial: mostra um traço enquanto prepara a leitura. Gráficos começam vazios. Download/upload são a soma das interfaces, incluindo virtuais/VPN, e podem contabilizar um mesmo tráfego em mais de uma interface. Não são um teste de velocidade da internet. Taxas usam KiB/s e MiB/s; RAM usa GiB.

Não há temperatura, GPU, clima, reprodução multimídia, galeria, dock ou calendário de eventos nesta versão. Não há valores fictícios para esses recursos. Toque/caneta, redimensionamento pelas bordas, múltiplas cópias do mesmo tipo, Win+D e DPI misto ainda não têm validação completa. Cada tipo pode ter uma janela ativa. O painel pode permanecer oculto na bandeja.

## Configurações e notas

Tudo é salvo em `%APPDATA%\dev.liquiddesktop.app\settings.json`. O schema 2 contém configurações por widget. As preferências da versão 0.1.x são migradas para o relógio digital, preservando posição, material e opções. Antes da primeira gravação, o arquivo anterior é copiado para `settings-before-widgets.json`. Gravações usam arquivo temporário e renomeação.

Notas são texto simples local, sem sincronização nem formatação Markdown. Aguarde o indicador **Salvo neste computador** antes de encerrar. Preferências inválidas geram erro e não são sobrescritas automaticamente. Para restaurar padrões, feche o aplicativo e renomeie `settings.json`, guardando a cópia se houver notas importantes.

## Desenvolvimento e compilação

Pré-requisitos: Windows 10/11, Node 22.12+ ou 24, Rust MSVC, Visual Studio Build Tools C++/Windows SDK e WebView2 Evergreen.

```powershell
npm ci
npm run tauri dev
npm test
npm run tauri build
```

Nesta pasta, `scripts/dev.ps1` e `scripts/build.ps1` configuram o Rust local de `.tools` quando disponível. O instalador NSIS não tem assinatura de código.

Testes Rust: `cargo test --manifest-path src-tauri/Cargo.toml`. Testes nativos/contadores reais são opt-in: acrescente `-- --include-ignored`. Eles criam apenas janelas ocultas temporárias e exigem acesso ao desktop/cursor. `liquid-desktop.exe --diagnose` grava um relatório local de wallpaper sem imagens ou caminhos pessoais.

## Sair, desinstalar e restaurar

Use Sair do Liquid Desktop no menu da bandeja para encerrar todas as janelas. Desinstale em **Configurações > Aplicativos > Liquid Desktop**. O app não altera wallpaper ou arquivos de sistema; a inicialização com Windows é opcional. Para remover também preferências e notas, após fechar exclua somente a pasta de configuração acima.

Consulte [arquitetura](docs/ARCHITECTURE.md), [design](DESIGN.md), [pesquisa](docs/RESEARCH.md) e [validação](docs/VALIDATION.md). A coleção é uma base funcional incremental; não representa a conclusão de todas as fases do projeto original.
