# Liquid Desktop 0.11.1

## Barra Principal e cantos discretos — 0.11.1

O modo **Principal** agora esconde a barra original do Explorer no monitor escolhido enquanto a barra Liquid estiver ativa; as barras dos outros monitores permanecem. Um processo auxiliar acompanha o aplicativo e restaura a barra original se ele fechar inesperadamente. Desativar a barra, mudar para Coexistir/Sobrepor ou sair normalmente também a restaura. A atualização migra uma vez o modo Coexistir e o raio padrão antigos para Principal e raio 10; depois disso, suas escolhas permanecem. O Explorer e os arquivos do Windows continuam intactos. A barra Liquid ainda não reproduz os ícones de terceiros da bandeja do sistema.

## Barra de tarefas Liquid Glass — 0.11.0

Ative **Barra de tarefas** no painel. A nova superfície nativa mostra aplicativos abertos e atalhos fixados, ícones obtidos do Windows quando disponíveis, indicação de foco/minimização e múltiplas janelas. Clique para abrir, focar ou minimizar; clique com o botão direito para abrir outra janela, fechar ou fixar/desafixar. Arraste os ícones fixados para reordenar. Atalhos `.lnk` já fixados na barra do Windows são importados; também é possível fixar um app aberto ou informar o caminho de um `.exe`/`.lnk` no painel.

A barra tem segmentos de Iniciar/Pesquisa/Visão de tarefas, aplicativos e status. Hora, data, conectividade, bateria e volume vêm do Windows; o indicador de mídia usa GSMTC. O clima opcional usa a cidade configurada no widget Clima e a Open-Meteo. Notificações e controles rápidos usam os atalhos nativos do Windows; o botão de áudio abre o Control Center do Liquid Desktop. Ícones de terceiros da área de notificação não são lidos, pois o Windows não oferece uma API pública estável para reproduzir a bandeja completa.

**Coexistir** fica acima da barra original, respeitando a área útil do monitor. **Sobrepor** fica na borda inferior e permite ocultar automaticamente a barra do Windows. **Principal (experimental)** ocupa a borda inferior e esconde a barra original enquanto o Liquid Desktop estiver ativo. O Explorer permanece ativo. A barra original volta ao desativar o modo, sair ou após uma falha do aplicativo. O app não troca o shell, não injeta código no Explorer e não modifica arquivos do sistema.

Escolha monitor, largura, altura, raio, tamanho/ampliação dos ícones, ordem e visibilidade dos módulos, ocultação automática e preset de material. **Mesmo material dos widgets** usa a lente WebGL2 compartilhada, inclusive frames do Wallpaper Engine. O contorno nativo acompanha os três segmentos: cliques nos espaços vazios passam para o aplicativo atrás. Tela cheia oculta a barra quando configurado. A 0.11.0 cria uma barra em um monitor por vez; apps protegidos, janelas sem título e ícones sem representação acessível podem não aparecer. Atalhos `.lnk` e o executável aberto podem aparecer separadamente quando o Windows não fornece um vínculo confiável entre eles.

## Correção de arraste e vidro — 0.10.1

Os quatro widgets adicionados na 0.10.0 agora estão incluídos na permissão de janelas do Tauri. Isso libera a leitura da posição necessária para o vidro óptico e elimina `plugin:window|outer_position not allowed by ACL`. Media Player e Galeria voltaram a permitir arraste pelo fundo; apenas seus botões, entradas e controles bloqueiam o gesto. A ordem nativa observada coloca os widgets acima do `WorkerW` do desktop.

## Coleção de widgets — 0.10.0

Agora há nove widgets: relógios digital e analógico, calendário, notas, sistema, Media Player, clima, galeria e armazenamento. Ative cada um em **Widgets**; posição, tamanho, material e preferências são salvos. **Restaurar configurações padrão** conserva fotos, notas, eventos e a posição atual.

- **Media Player:** reutiliza as sessões de mídia da ilha. Mostra capa, faixa, artista, aplicativo, progresso e controles. O medidor opcional lê o pico real da saída de áudio do Windows. Há layouts compacto, horizontal e expandido.
- **Clima:** busque uma cidade no painel. A previsão vem da Open-Meteo, com unidades e intervalo de atualização configuráveis. Uma falha de rede aparece no widget; não há previsão simulada.
- **Galeria:** importe fotos ou uma pasta pelo widget. As imagens são copiadas para os dados locais do app; o limite é de 120 arquivos e 25 MB por arquivo. Ajuste intervalo, ordem, enquadramento e moldura de vidro.
- **Armazenamento:** mostra capacidade, uso e espaço livre das unidades que o Windows relata; escolha unidades no painel. Sem seleção, exibe todas.
- **Sistema, calendário e notas:** seleção das métricas e layout do sistema, frequência de CPU quando disponível, eventos locais no calendário e prévia segura de Markdown nas notas. Sensores de GPU e temperaturas não são mostrados sem uma fonte confiável; lembretes ficam no calendário e ainda não geram notificações do Windows.

O motor óptico, a Top Bar e a Dynamic Island continuam os mesmos. O vidro dos widgets novos usa a mesma captura do Wallpaper Engine e os mesmos controles de material.
## Notch sobre aplicativos — 0.9.2

A ilha Liquid/Adaptive e a Top Bar agora recebem os pixels dos aplicativos que estão atrás delas no monitor capturado, inclusive janelas de tema escuro. Os widgets continuam mascarando aplicativos para permanecerem como superfícies de desktop. Quando a exclusão da própria janela produz um grande bloco branco no frame DXGI, esse bloco fica transparente antes da refração, inclusive no notch expandido. Não é possível mostrar conteúdo de janelas que o próprio Windows ou outro aplicativo protege da captura.

## Captura e notch — 0.9.1

Em alguns drivers do Windows 10, DXGI entrega blocos brancos opacos dentro das janelas transparentes excluídas da captura. Blocos grandes e quase totalmente brancos agora ficam transparentes na textura, preservando o vidro sem o quadrado opaco. Pequenos pontos claros e o restante da imagem permanecem intactos. A ilha Liquid/Adaptive posiciona o canvas óptico na mesma camada usada pela barra e pelos widgets; os modos Black e Graphite continuam opacos quando escolhidos.

## Painel redesenhado — 0.9.0

O painel agora usa navegação lateral para Visão geral, Widgets, Dynamic Island, Top Bar, Materiais, Aparência e Aplicativo. A tela de widgets mantém todas as configurações e mostra uma prévia contextual. O Material Studio separa presets, prévia com o renderer óptico real e controles por função. A aparência do painel usa neutros minerais e verde discreto; o vidro permanece concentrado nas prévias e no desktop. Preferências e comandos nativos permanecem compatíveis.

## Correções do vidro animado — 0.8.1

A composição da cor e da transparência agora é idêntica entre pixels capturados e áreas mascaradas, eliminando a emenda retangular de cor. Barra e ilha usam uma lente proporcional à altura, preservando o centro transparente e a refração nas bordas. Encerre a versão anterior pela bandeja antes de instalar: uma sessão antiga de captura mantém seu indicador amarelo até ser encerrada.

## Refração animada e material compartilhado — 0.8.0

Ative **Fundo animado / Wallpaper Engine** em Material. Os frames capturados alimentam a mesma lente WebGL2 usada no modo estático: refração, dispersão RGB, difusão e reflexos. Widgets individuais, widgets unidos, Top Bar e ilha Liquid/Adaptive agora usam esse mesmo renderizador e os mesmos parâmetros. Barra e ilha não têm mais uma cor cinza independente.

Na primeira migração, o material de um widget ativo é compartilhado com os demais; posições, tamanhos e notas permanecem individuais. A configuração anterior é copiada para `settings-before-live-glass.json` antes da primeira gravação. **Sincronizar vidro dos widgets, barra e ilha** mantém o conjunto sincronizado. Desativar permite materiais individuais; a barra/ilha seguem o último material ajustado. Black e Graphite da ilha continuam sendo alternativas opacas.

A versão 0.8.1 usa DXGI Desktop Duplication no monitor do Wallpaper Engine, sem iniciar sessões WGC que produziam a borda amarela. As imagens permanecem em memória, e os recortes enviados ao renderer têm as áreas dos aplicativos mascaradas. Onde outro aplicativo cobre o fundo, o vidro volta à translucidez: não reconstrói wallpaper oculto. A máscara é conservadora e pode cobrir margens/sombras de janelas.

As próprias janelas do Liquid Desktop são excluídas da captura para evitar recursão. **Enquanto o modo animado estiver ativo, elas também podem não aparecer em screenshots/gravações que respeitam essa exclusão do Windows.** Ao desligar todas as superfícies animadas, a exclusão é removida. Nada é enviado pela rede nem gravado em vídeo.

O fundo precisa continuar reproduzindo no Wallpaper Engine; regras de pausa ao abrir outro app também pausam o efeito. A cadência solicitada é de até 30 atualizações/s (10 no Econômico), sem promessa de FPS medido. A captura é compartilhada por monitor e encerra sessões sem consumidores após dois segundos. Perda de dispositivo/Wallpaper Engine é sinalizada e há nova tentativa.
## Nitidez, barra e ilha — 0.7.1

O vidro unido agora acompanha a resolução física do monitor; Alta acrescenta supersampling de 1,5×, limitado pela GPU. Removido o limite de textura de 2048 px; a borda usa antialiasing derivado do pixel e o recorte nativo usa linhas de um pixel. Mapas dos widgets individuais também acompanham DPI (até 2×).

A Top Bar usa o shader óptico com cor e intensidade configuráveis. Na ilha, selecione **Aparência → Liquid Glass** (ou Adaptive), tanto no formato flutuante quanto notch. O vidro acompanha a expansão e mantém o texto fora do efeito. Black e Graphite continuam disponíveis.

A fonte do fundo agora é compartilhada pelo painel Material. A reserva de espaço e o recorte central da barra permanecem iguais.
## Vidro líquido e widgets unidos — 0.7.0

No painel, selecione um widget e clique em **Widgets unidos → Usar vidro líquido de…**. O grupo usa esse widget como fonte de cor e material. Ative **Permitir arrastar** em cada widget: ao aproximá-los, as bordas formam uma lente contínua, com reflexos e dispersão RGB. As posições do grupo são salvas separadamente; desativar a união restaura as janelas nas posições individuais anteriores. Notas e configurações são preservadas. Um monitor por grupo nesta versão.

O modo unido refrata tanto o wallpaper estático quanto os frames animados disponíveis. O texto permanece fora do shader. Não usa Acrylic. A qualidade controla a resolução da superfície; o consumo total de GPU ainda precisa ser medido no desktop.

[Prévia do renderizador numa cena de teste](docs/images/fusion-preview.png). A imagem usa uma textura ilustrativa; a versão instalada usa o wallpaper estático do Windows.

## Control Center — primeiro incremento

Clique no ícone de áudio da Top Bar. Escolha um dispositivo ativo para ajustar seu volume/mute, consulte conectividade e energia reais e abra as configurações correspondentes. A escolha no painel controla aquele dispositivo; para mudar a saída padrão, use **Escolher saída padrão no Windows**. Esc ou perder o foco fecha o flyout. Informações indisponíveis são indicadas explicitamente.

## Espaço reservado para aplicativos — 0.6.1

Ativar a Top Bar ou a ilha agora registra uma AppBar do Windows no monitor escolhido. Aplicativos maximizados que respeitam a área útil ficam abaixo da faixa reservada. No mesmo monitor, barra e ilha compartilham uma única reserva, sem somar alturas. Em monitores diferentes, cada um recebe sua própria reserva.

Na 0.6.2, a margem abaixo da superfície persistente foi reduzida para 2 unidades lógicas (6 no modo compacto de música). Avisos temporários e expansão podem avançar sobre os aplicativos, sem mover a área útil. A faixa é estável: 46 unidades lógicas para a barra, 70 para ilha flutuante ou 58 para notch, convertidas conforme o DPI. A ilha expandida abre sobre os aplicativos; expandir/recolher não redimensiona as janelas. Desativar as superfícies ou sair pela bandeja libera o espaço. A negociação respeita outras AppBars e a barra de tarefas; o registro é refeito após a notificação de reinício do Explorer.

Janelas posicionadas manualmente e aplicativos que ignoram a área útil podem continuar sobrepondo a faixa. Tela cheia real continua usando o monitor inteiro e a ocultação das superfícies permanece disponível. Não são alteradas configurações globais de resolução nem a área útil via registro/SPI_SETWORKAREA.

## Top Bar — primeiro incremento

No painel, ative **Top Bar**. A barra mostra hora/data, CPU, RAM, download/upload e volume real do Windows. Clique no ícone à esquerda para abrir o painel; o botão de áudio abre o Control Center e o slider ajusta o volume da saída padrão. As métricas compartilham o provedor do widget Sistema, com atualização a cada 2 segundos; leituras indisponíveis aparecem como travessão.

Escolha monitor, módulos visíveis, segundos, camada acima dos aplicativos e ocultação em tela cheia. Cor e demais parâmetros do vidro vêm do painel Material. Uma barra pode estar ativa neste incremento, com altura de 36 unidades lógicas e margem de 8. Se o monitor salvo desconectar, usa o primeiro disponível.

Quando a ilha está no mesmo monitor, a barra se divide em dois segmentos e reserva 480 unidades no centro. Esse espaço também é retirado da região nativa da janela, deixando os cliques passarem. Em segmentos estreitos, rede, slider, RAM e outros detalhes se ocultam progressivamente; o painel continua acessível. Monitores com menos de 496 unidades lógicas não comportam a reserva central e os segmentos da barra. Desative a ilha ou use outro monitor nesse caso.

A Top Bar começa desligada ao atualizar a partir de layouts sem barra. Quando ativa, reserva espaço pela AppBar e preserva a barra de tarefas do Windows. O material é translúcido, sem Acrylic. Reordenação de módulos e múltiplas barras simultâneas ficam para próximos incrementos.

## Correção do vidro — 0.5.2

Retirado o Acrylic nativo da ilha: na 0.5.1 ele desenhava um retângulo de 480×340 fora do contorno. A ilha mantém transparência, reflexos direcionais e animações suaves. A opção de desfoque nativo foi removida; configurações antigas com ela ativada são ignoradas sem perder notas, posições ou aparência. Encerre a versão anterior pela bandeja antes de abrir a nova.

O material atual da ilha não desfoca nem refrata o conteúdo de outros aplicativos. A refração dos widgets sobre wallpaper estático continua disponível. Um backdrop nativo só deve voltar após validação visual do recorte no desktop.

## Dynamic Island — primeiro incremento funcional

No painel, ative **Dynamic Island**. Escolha Floating Island ou Notch, aparência Black/Graphite/Liquid e monitor. A opção **Acima de aplicativos** vale apenas para a ilha; os widgets continuam na camada do desktop. A ilha começa desligada ao atualizar.

Passe o cursor para expandir; clique no fundo ou em Fixar para mantê-la aberta. Esc recolhe. A geometria muda com mola interrompível, e o contorno nativo limita a área clicável. A janela é ocultada ao detectar uma aplicação cobrindo todo o monitor (verificação de 1 segundo, ainda requer validação com jogos e DPI misto).

Mídia usa GSMTC: título, artista, capa quando fornecida, seleção de sessão, progresso e comandos suportados pelo player. Volume/mute usam a saída multimídia padrão do Windows via Core Audio. Não existe visualizador de áudio artificial. Players que não publicam sessões não aparecem; falhas são indicadas na ilha.

O temporizador funciona enquanto o app está aberto, usa relógio monotônico e notifica na própria ilha ao terminar. Não é um alarme persistente: desativar a ilha ou encerrar o aplicativo cancela o temporizador. Mídia e timer podem aparecer simultaneamente. Aviso de conclusão tem prioridade sobre aviso de volume por 4,5 segundos.

Liquid/Adaptive usam o mesmo material óptico dos widgets, com as limitações de captura descritas acima. A ilha ainda não possui fusão entre atividades. Notificações de terceiros, controles de microfone e Dock continuam pendentes. Esses itens continuam no ROADMAP.

O plano [Next Generation](ROADMAP.md) começou pelo motor óptico da 0.4.0. Os cinco widgets, bandeja, temas sincronizados e inicialização opcional continuam disponíveis.

## Motor óptico e materiais

No painel, selecione um widget e escolha **Liquid Prism**, **Clear**, **Thin**, **Regular**, **Frosted**, **Dark** ou **Adaptive**. A prévia é um estúdio com textura ilustrativa, processada pelo mesmo renderer usado no desktop. Cor e material continuam podendo ser sincronizados entre widgets.

Novos ajustes: espessura óptica, largura da borda convexa, dispersão cromática, difusão da textura e qualidade. O centro da lente permanece neutro e a refração se concentra na borda. O contraste configurado é compartilhado entre superfícies; o conteúdo fica fora da refração. O modo animado usa captura em memória conforme descrito acima.

Econômico desliga a dispersão e reduz a cadência de captura. Equilibrado acompanha o DPI físico; Alta acrescenta supersampling de 1,5×, limitado pela GPU. A resposta elástica usa uma mola interrompível e respeita movimento reduzido. [Detalhes de movimento](docs/MOTION.md). Preferências 0.3.0 são carregadas como material Personalizado, sem substituir cores, posições, notas ou valores anteriores.

A conferência visual no WebView2 permanece pendente por indisponibilidade da automação de interface neste ambiente. Testes e benchmark isolado constam em [validação](docs/VALIDATION.md); não representam uma medição de FPS ou de consumo total do aplicativo.

## Bandeja, temas e inicialização

Minimizar ou fechar o painel mantém os widgets ativos na bandeja do sistema. Clique no ícone para reabrir; clique com o botão direito e escolha **Sair do Liquid Desktop** para encerrar. Uma segunda abertura do app reaproveita a instância existente.

Em **Material**, ative **Sincronizar vidro dos widgets, barra e ilha** para aplicar o material selecionado à coleção inteira, inclusive widgets ocultos. Cor, intensidade, reflexos, contraste, refração e modo de wallpaper são compartilhados; posições, tamanhos, notas e opções dos relógios continuam individuais. Desativar a sincronização permite editar cada um novamente. Temas prontos: Cristal, Grafite, Esmeralda e Âmbar.

**Iniciar com o Windows** registra o executável instalado apenas para o usuário atual em HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run. Ao entrar no Windows, restaura os widgets sem abrir o painel. Ative essa opção na versão instalada, não em uma compilação de desenvolvimento. A opção começa desativada e não é ligada pelo instalador. Se bloquear a inicialização no Gerenciador de Tarefas, reative-a também lá. Desative a opção antes de desinstalar para remover a entrada de inicialização.

Widgets nativos para o desktop Windows, com Tauri 2, Rust, React/TypeScript e vidro compartilhado. Cada widget é uma janela transparente independente, sem moldura, posicionada acima do desktop e abaixo de aplicativos. Compatível com a captura do Wallpaper Engine visível no desktop; não é uma página simulando um desktop.

## Usar

Encerre a versão antiga pelo menu **Sair do Liquid Desktop** na bandeja antes de instalar a atualização. O instalador fica em `src-tauri/target/release/bundle/nsis/Liquid Desktop_0.10.1_x64-setup.exe`.

No painel, escolha um widget na coleção e clique **Adicionar**. Selecione o card para configurar apenas aquele widget. **Remover** fecha sua janela e mantém suas preferências. Widgets ativos são reabertos ao iniciar o app.

| Widget | Funcionalidade |
|---|---|
| Relógio digital | Hora e data locais; segundos opcionais; formato 12/24 horas |
| Relógio analógico | Ponteiros reais; ponteiro de segundos opcional |
| Calendário | Mês atual, navegação entre meses e botão Hoje; não acessa agendas |
| Notas rápidas | Texto simples editável no desktop, salvo localmente; até 8.000 caracteres |
| Monitor do sistema | CPU, RAM, download e upload reais; gráficos das últimas 30 amostras |

Cada widget tem posição X/Y em pixels físicos, largura e altura em unidades lógicas, cor, intensidade da cor, reflexos, contraste, refração e modo de edição próprios. O tamanho pode ser alterado pelo painel (200–640 × 180–600); **Usar tamanho sugerido** restaura uma composição adequada ao tipo.

Ative **Permitir arrastar**. Arraste pelo fundo ou título do widget. Esc cancela; soltar salva a posição. Botões do calendário e a área de texto das notas continuam interativos e não iniciam arraste. A moldura nativa é removida via tratamento Win32 na thread da janela, preservando a correção confirmada na versão 0.1.4.

## Wallpaper Engine e vidro

**Fundo animado / Wallpaper Engine** vem ligado por padrão. Usa transparência, cor, bordas e reflexos, sem copiar a imagem estática antiga para dentro do widget. Não há refração do vídeo nem captura contínua de tela.

Para um wallpaper estático do Windows, desmarque a opção. `IDesktopWallpaper` fornece a imagem real por monitor, que é mapeada e refratada por SVG. O mapa acompanha o tamanho do widget. Consulte mudanças a cada 15 segundos somente nesse modo. Para wallpapers claros, aumente **Contraste de leitura**.

## Dados reais e limites

O monitor usa `sysinfo` 0.37.2 com apenas os recursos de sistema/rede. Atualiza a cada 2 segundos enquanto o documento está visível. CPU precisa de uma amostra inicial: mostra um traço enquanto prepara a leitura. Gráficos começam vazios. Download/upload são a soma das interfaces, incluindo virtuais/VPN, e podem contabilizar um mesmo tráfego em mais de uma interface. Não são um teste de velocidade da internet. Taxas usam KiB/s e MiB/s; RAM usa GiB.

Não há temperatura, GPU, clima, galeria, dock ou calendário de eventos nesta versão. O controle de mídia está disponível na Dynamic Island, para players que expõem sessões GSMTC. Não há valores fictícios para esses recursos. Toque/caneta, redimensionamento pelas bordas, múltiplas cópias do mesmo tipo, Win+D e DPI misto ainda não têm validação completa. Cada tipo pode ter uma janela ativa. O painel pode permanecer oculto na bandeja.

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
