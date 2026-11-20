# Validação

## Versão 0.6.2 — margem compacta

- Build release e instalador NSIS concluídos: `Liquid Desktop_0.6.2_x64-setup.exe` (2,24 MiB).
- Passaram os 2 testes Rust de cálculo da reserva, com expectativas atualizadas para os novos limites e DPI 150%.
- Reserva reduzida para 46 unidades lógicas com barra, 70 com ilha flutuante e 58 com notch. São 2 unidades abaixo da maior superfície persistente, ou 6 abaixo da ilha no modo de música. Avisos transitórios podem avançar sobre os aplicativos; hover continua sem alterar a reserva.
- TypeScript/Vite passaram. A API AppBar e seu ciclo de registro/liberação não mudaram; o teste de alteração/restauração real da 0.6.1 não foi repetido para evitar mexer novamente nas janelas do usuário.
- Inspeção visual da distância final permanece pendente no desktop instalado.

## Versão 0.6.1 — AppBar e área útil

- Passaram 27 testes JavaScript e 13 testes Rust não opt-in. Novos testes cobrem combinação de alturas sem soma, monitores diferentes, fallback de monitor, superfície ausente/desativada e conversão por DPI.
- Teste nativo de AppBar passou fora do sandbox: registra uma janela auxiliar, confirma via GetMonitorInfo que a área útil do Windows diminui, preserva a barra inferior e confirma restauração exata depois de ABM_REMOVE/Drop. Dentro do sandbox, o Explorer recusou ABM_NEW; a repetição autorizada fora dele passou. A alteração de teste é temporária e protegida por RAII.
- A verificação ampliada também passou: reserva de 8 → 16 → 8 pixels, manutenção repetida sem acumular espaço e liberação final. Total de 14 testes Rust distintos executados nesta entrega.
- Build Rust release, TypeScript/Vite e instalador NSIS concluídos: `Liquid Desktop_0.6.1_x64-setup.exe` (2,25 MiB).
- Pendente: reinício real do Explorer, múltiplos monitores/DPI físicos, fullscreen e comportamento de aplicativos específicos já maximizados. O teste de área útil não comprova que todo aplicativo de terceiros a respeita. A ilha expandida não altera a altura reservada.

## Versão 0.6.0 — Top Bar, primeiro incremento

- Build Rust release e instalador NSIS concluídos: `Liquid Desktop_0.6.0_x64-setup.exe` (2,23 MiB, 2.334.079 bytes).
- Passaram 27 testes JavaScript e 11 testes Rust não opt-in. Cobertura nova: contadores indisponíveis versus zero real, unidades de rede/RAM, tint/alpha, migração de layout, validação de configuração, monitor desconectado, origem negativa e DPI 150%.
- Passou também o teste nativo opt-in da Top Bar: um HWND oculto confirma por GetWindowRgn/PtInRegion que o vão central fica fora da região, os dois segmentos recebem cliques, os cantos são arredondados e o centro volta a pertencer à janela quando a reserva é removida. Inclui 150% de DPI e largura menor que o vão. Total nesta versão: 12 testes Rust executados; os 5 opt-in antigos não foram repetidos.
- TypeScript/Vite passaram. Os provedores de sistema e áudio foram reaproveitados; não houve alteração intencional de reprodução ou volume durante os testes.
- Pendente: inspeção visual WebView2 da barra e painel, comandos interativos de volume/mute, transparência sobre fundos variados, monitores/DPI reais, fullscreen/jogos, Win+D e benchmark. Testes de região nativa não equivalem à inspeção visual do compositor.
- Escopo entregue: uma barra, módulos fixos com visibilidade configurável, sem Acrylic, sem AppBar ou Control Center. Top Bar começa desativada; layouts anteriores preservam widgets e ilha.

## Versão 0.5.2 — regressão de Acrylic

- Build release e instalador NSIS concluídos: `Liquid Desktop_0.5.2_x64-setup.exe`.
- Screenshot do usuário confirmou que Acrylic pintava todo o HWND 480×340 apesar do recorte da ilha. A validação anterior por compilação/testes não demonstrava recorte visual correto.
- Removidos aplicação de Acrylic, comando IPC, dependência direta e opção de interface. Nova janela usa apenas material translúcido CSS e região nativa; animação e reflexos continuam.
- Passaram 24 testes JavaScript e 9 Rust; 5 testes nativos/hardware opt-in não foram executados. Novo teste carrega configuração 0.5.1 com blur ativado, preserva notas/posição/aparência e confirma que o campo legado não é salvo novamente.
- TypeScript/Vite passaram. Validação visual do novo instalador no desktop ainda pendente. O processo anterior precisa ser encerrado para que seu backdrop seja destruído.

## Versão 0.5.1 — movimento e vidro nativo

- Build release e instalador NSIS concluídos: `Liquid Desktop_0.5.1_x64-setup.exe` (2,20 MiB).
- Passaram 24 testes JavaScript, incluindo equivalência da trajetória em 30/60/144 Hz, inversões rápidas com conservação de velocidade e continuidade da geometria do cabeçalho. Isso valida matemática, não FPS real.
- Passaram 8 testes Rust; 5 testes opt-in de desktop/hardware não foram repetidos nesta versão. Migração preserva dados anteriores e a preferência de blur desligado.
- TypeScript/Vite e cargo check passaram. Desfoque usa `window-vibrancy` já presente transitivamente, agora como dependência direta para tratar erros.
- Pendente: inspeção visual no WebView2, contraste com fundos claros, Acrylic sobre Wallpaper Engine/aplicativos, clipping durante inversões rápidas, DPI misto e custo GPU. A indisponibilidade anterior do helper nativo não foi resolvida neste incremento. Não há afirmação de refração ao vivo ou FPS medido.

## Versão 0.5.0 — Dynamic Island, primeiro incremento

- Build Rust release e instalador NSIS concluídos: `Liquid Desktop_0.5.0_x64-setup.exe` (2,19 MiB). SHA-256: `19e26093f12d49d6c662812a39ecfd33f2d7065939d068ef2bcb816351ec84be`.
- Passaram 21 testes JavaScript: arbitragem de atividade, expiração/prioridade de avisos, coalescência de geometria, mola e regressões anteriores.
- Passaram 13 testes Rust, incluindo todos os opt-in nativos: a região da ilha exclui margens, cobre a expansão, preserva o topo conectado do notch e volta à cápsula. Moldura, arraste e ordem dos widgets anteriores passaram.
- Leitura GSMTC real: duas sessões, com faixa ativa. Leitura real do endpoint de áudio passou. Teste não executou play/pause/seek nem alterou volume/mute. Falhas de sessão inexistente foram rejeitadas.
- Migração mantém notas e posições e adiciona ilha desligada; configuração inválida de monitor é rejeitada. TypeScript/Vite passaram.
- Pendente: inspeção visual WebView2, gestos rápidos durante expansão, comandos de mídia nos players, fullscreen/jogos, DPI misto, mudança de monitores, Win+D/Alt+Tab e benchmark CPU/GPU/RAM. O helper de UI estava indisponível na validação anterior; testes de HWND oculto não equivalem à validação visual.
- Não implementado: metaball merging, medição/visualizador de áudio, notificações de terceiros, seleção de endpoint/microfone, Top Bar e Dock. Não há alegação de 60 FPS medidos.

## Versão 0.4.0 — primeiro incremento do motor óptico

- Build Rust release e instalador NSIS concluídos: `Liquid Desktop_0.4.0_x64-setup.exe` (2,13 MiB), com aviso de licença QuickLiquid incluído nos recursos.

- Passaram 17 testes JavaScript (incluindo neutralidade do centro, simetria, refração para dentro, limites de resolução, luminância e continuidade/convergência da mola).
- Passaram 10 testes Rust, incluindo todos os testes nativos/hardware opt-in fora do sandbox: moldura/arraste, ordem de múltiplos widgets, contadores reais e migração óptica sem perda de notas/geometria. O teste de migração valida campos inválidos e sincronização dos novos parâmetros.
- TypeScript e Vite passaram. Não foram habilitados novos privilégios IPC.
- Benchmark reprodutível: `node scripts/benchmark-optics.mjs`, Node 24.15/Windows, 40 amostras após aquecimento. Somente rasterização CPU, excluindo PNG, WebView/GPU, IPC e composição.

| Tamanho | 0.3.0 mediana | 0.4.0 equilibrado mediana | Alta mediana |
|---|---:|---:|---:|
| 360×220 | 2,036 ms | 0,805 ms | 0,808 ms |
| 380×340 | 3,269 ms | 1,121 ms | 1,125 ms |
| 640×600 | 8,795 ms | 1,019 ms | 2,113 ms |

Não extrapolar esses números para FPS. Custo de três passagens RGB, memória de múltiplos WebViews e consumo total antes/depois ainda precisam de medição no aplicativo instalado.

Bloqueio de conferência visual: CUA retornou inventário vazio; Computer Use retornou `native pipe unavailable (os error 2)`. Não foi substituída essa validação por alegações de screenshots. A prévia usa o renderer real, mas ainda precisa de inspeção no WebView2.

Checklist pendente: comparar os seis materiais em fundo claro/escuro, arrastar e cancelar com Esc em modo estático, alternar Wallpaper Engine sem textura antiga, reduzir movimento, redimensionar nos limites, DPI misto e cinco widgets ativos. Preservar a fase visual como pendente até essa conferência. Fusão/metaballs e morphing da ilha não foram implementados neste incremento.

## Versão 0.3.0 — bandeja, temas e inicialização

- Build Rust release e instalador NSIS concluídos: `Liquid Desktop_0.3.0_x64-setup.exe` (2,11 MiB).

- Passaram 11 testes JavaScript e 6 testes Rust. O novo teste verifica sincronização do material para todos os tipos, preservação de conteúdo/geometria/estado ativo e persistência da opção. Layouts antigos recebem sincronização desativada.
- TypeScript/Vite passaram. Os 3 testes nativos/hardware opt-in não foram repetidos nesta versão; os resultados anteriores continuam registrados abaixo.
- Bandeja e instância única compiladas com Tauri. A inicialização grava apenas a chave Run do usuário atual; não foi ativada neste ambiente de desenvolvimento.
- Pendente: validar interativamente minimizar/fechar/reabrir/sair pela bandeja e entrada no Windows com o aplicativo instalado. Não foi realizado logoff ou reinício da sessão do usuário.

## Versão 0.2.0 — coleção de widgets

- Relógio digital e novos widgets analógico, calendário, notas e monitor de CPU/RAM/rede. Configuração e persistência independentes por tipo.
- Passaram 11 testes JavaScript: geometria, sessão de arraste e calendário (incluindo ano bissexto e mudança de ano).
- Passaram 8 testes Rust, incluindo migração das preferências antigas, limites e persistência de configurações/notas, janela sem moldura, movimento, ordem estável entre múltiplos widgets e leitura real de memória/CPU/rede. Testes nativos e de hardware executados fora do sandbox.
- TypeScript/Vite e build de produção Rust/NSIS concluídos. Executável e instalador posteriores à alteração da ordem das janelas.
- Instalador: `Liquid Desktop_0.2.0_x64-setup.exe`, 2.133.136 bytes.
- SHA-256: `09ACAA0BA345CC8092FD8EC7F25D5DE4D27FC4BB6FED2A6A61F885237F1CA5C2`.
- O usuário confirmou o funcionamento da correção de moldura da 0.1.4. Essa implementação foi preservada.
- Pendente: inspeção visual dos novos widgets no WebView2, instalação/atualização interativa e uso conjunto com Wallpaper Engine. A automação nativa de interface não estava disponível; os testes de HWND não substituem essa inspeção. DPI misto, Win+D e gerenciadores de janela externos continuam sem validação completa.

As seções abaixo são registros históricos das versões anteriores.

## Versão 0.1.4 — cálculo e pintura da moldura

- Passaram os 3 testes Rust, incluindo a regressão que restaura WS_CAPTION e força SWP_FRAMECHANGED: a origem do cliente permanece igual à origem externa e sua largura/altura cobrem toda a janela. A barra reserva zero pixels no HWND de teste.
- O teste mantém a verificação de arraste e retorno à origem. Não substitui teste visual com WebView2 e o tema/gerenciador de janela do usuário.
- TypeScript/Vite, Rust release e NSIS concluídos.
- Instalador: `Liquid Desktop_0.1.4_x64-setup.exe` (1,96 MiB).
- SHA-256: `66E514632E58C86A04C95D685B3453E637A125BA94AB22B0755CA9A8FA34C060`.
- A versão 0.1.3 foi refutada pela captura enviada pelo usuário: os testes de flags não detectavam a faixa persistente. A verificação geométrica da 0.1.4 cobre esse caso de forma mais direta.

## Versão 0.1.3 — moldura e material

- Passaram os 8 testes JavaScript e 3 testes Rust (incluindo regressão nativa de restauração/remoção de WS_CAPTION, botões e borda; movimento sem pai; migração e validação de cor/reflexos/contraste).
- TypeScript, Vite, Rust release e instalador NSIS concluídos.
- Cor hexadecimal, amostras, reflexos/espessura e contraste persistentes. Instrução oculta fora da edição.
- Efeito óptico refinado por borda mascarada e reflexos internos; não há refração do vídeo do Wallpaper Engine.
- A aparência no desktop e o comportamento de gerenciadores de janela externos ainda dependem de inspeção visual.
- Instalador: Liquid Desktop_0.1.3_x64-setup.exe (1,96 MiB). SHA-256: D1060DE4FFC89725F10295CE61377415CADD83AFA3A7F75A9425C9D37E00BB94.


## Versão atual: 0.1.2 — janela independente

- Corrigida a condição que exigia pai válido para mover um HWND top-level. Não há mais SetParent ou ScreenToClient no caminho de criação/movimentação.
- **Passou:** teste nativo `independent_widget_moves_without_explorer_parent`, fora do sandbox: comprova parent NULL, leitura de cursor, deslocamento e retorno à origem. No sandbox, somente a leitura de cursor foi bloqueada.
- **Passou:** 8 testes JavaScript, build TypeScript/Vite, Rust release e NSIS.
- Modo fundo animado ativado por padrão, sem consultar/carregar textura estática. A composição mantém transparência; não refrata vídeo.
- A ordem de janelas é recalculada uma vez por segundo para posicionar o widget acima de Progman/WorkerW e abaixo de aplicativos. **Ainda não validado visualmente com Wallpaper Engine**, outros substitutos do shell, Win+D ou múltiplos DPI.
- Instalador: `src-tauri/target/release/bundle/nsis/Liquid Desktop_0.1.2_x64-setup.exe`, aproximadamente 1,95 MiB.
- SHA-256: `D8FD99A0D260876F973D0EB1C9B9D7F4EC2EFA2777DFE5320CB36AF82831000B`.

O histórico abaixo registra resultados das versões anteriores; testes de HWND filho não validavam o caso sem pai relatado pelo usuário.

Este documento distingue implementação de comprovação. Não avançar para as outras fases antes de fechar o gate nativo.

## Automatizado

- `npm test`: geometria de fill, fit, stretch, center e span com origens negativas.
- `npm run build`: TypeScript e bundle Vite.
- `cargo check --manifest-path src-tauri/Cargo.toml`: backend e bindings.
- `npm run tauri build`: executável e instalador NSIS.

## Resultados locais — 16/11/2026

- **Passou:** `npm test`, 5 testes de geometria.
- **Passou:** TypeScript + Vite, bundle de produção.
- **Passou:** `cargo check` e compilação Rust MSVC de produção.
- **Passou:** instalador NSIS x64, aproximadamente 1,96 MiB, em `src-tauri/target/release/bundle/nsis/Liquid Desktop_0.1.0_x64-setup.exe`.
- **Passou:** diagnóstico nativo executado fora do sandbox: host elegível encontrado; 1 monitor 1920×1080; modo Preencher (4); textura real carregada; nenhum erro do provedor. O sandbox impedia inicialmente acesso COM com `0x80070005`.
- **Correção baseada no diagnóstico:** o host inicial Progman com DefView não existia neste ambiente; foi adicionada descoberta do WorkerW existente que contém DefView. A segunda execução confirmou `compatible_host: true`.
- **Não validado:** anexação/renderização/input visual do HWND. O helper Computer Use retornou `native pipe unavailable` (erro 2). Detectar host elegível não comprova interação, ordem de janelas ou DPI.
- **Não executado:** instalar/desinstalar via NSIS, testes de longa duração e matriz Windows 10/11.

SHA-256 do instalador desta entrega:

```text
F9D7A199E98D75C477AF6B9A42D613B947D7873076939FF4DC3D4769F70956C1
```

O gate da Fase 1 permanece aberto por falta de validação visual. As fases seguintes não foram iniciadas.

## Roteiro manual obrigatório

1. Abrir painel, adicionar relógio; verificar wallpaper real e ausência de fundo opaco.
2. Abrir e maximizar outro aplicativo: relógio deve permanecer atrás. Testar Alt+Tab, taskbar, Win+D e clique em arquivos fora dos limites e nos cantos arredondados.
3. Ativar edição, mover X/Y e verificar alinhamento da textura com linhas reconhecíveis do wallpaper, inclusive perto das bordas do monitor.
4. Alterar wallpaper e os seis modos; aguardar até 15 s. Usar imagens diferentes por monitor.
5. Usar dois monitores com origem negativa e DPI 100/150/200%; reiniciar o app; remover um monitor e recriar widget.
6. Reiniciar Explorer manualmente e verificar diagnóstico/recuperação por recriação, sem crash ou alteração permanente.
7. Inspecionar contraste, serrilhado e refração em WebView2; medir idle e animação com ferramentas de desempenho.
8. Instalar NSIS, executar, remover widget, sair e desinstalar. Confirmar que wallpaper, ícones e barra de tarefas continuam intactos.

Pendentes: verificação visual nativa, matriz Windows 10/11, múltiplos monitores físicos, FPS/consumo, acessibilidade e teste completo do instalador. Não há alegação de conclusão desses testes.

## Atualização — arraste direto

Compilação TypeScript/Vite, Rust release e NSIS concluídas com sucesso após adicionar o arraste. A interação por mouse permanece pendente de teste visual no desktop.

Roteiro adicional: ativar edição, arrastar pelo centro e pelas bordas, soltar fora da posição inicial, conferir X/Y no painel, recriar o widget para verificar persistência, cancelar com Esc, tentar arrastar com edição desativada e testar mouse com botões invertidos. Verificar textura durante movimento e múltiplos monitores com DPI diferente.

## Correções 0.1.1 — legibilidade e arraste

- 8 testes JavaScript passaram, incluindo ordenação início/soltura, cancelamento idempotente e liberação da sessão após erro.
- TypeScript, Vite, Rust release e NSIS passaram.
- Instalador: `src-tauri/target/release/bundle/nsis/Liquid Desktop_0.1.1_x64-setup.exe` (1,95 MiB).
- SHA-256: `A20D5EF42869706D067295AAC42BC1FDDA10F70AD0EE1F2E91C0DFDD8738EBFA`.
- Contraste calculado do texto branco sobre o pior caso da camada de legibilidade: 5,31:1, acima de 4,5:1. O cálculo não substitui inspeção visual.
- O helper Computer Use continua indisponível (`native pipe`, erro 2). O gesto completo dentro do WebView2 ainda precisa de verificação visual; não foi declarado aprovado.
- Adicionado teste nativo opt-in `desktop_child_moves_and_returns_to_origin`, que cria exclusivamente um HWND oculto temporário, anexa ao host existente, move, verifica as coordenadas e retorna à origem antes de destruir a janela de teste.
- **Passou fora do sandbox:** teste nativo de anexação, leitura do cursor, deslocamento físico e retorno à origem (1 teste, 0 falhas). No sandbox o teste não localizava o host do Explorer; a repetição autorizada teve sucesso. Esse resultado comprova as operações Win32 isoladas, não o gesto completo no WebView2.

## 0.7.0 — união óptica e Control Center (18/11/2026)

- Passaram 27 testes JavaScript e 15 testes Rust, incluindo união de superfícies próximas, lacunas entre distantes e migração com preservação de posições/notas. Nove testes de integração permanecem opt-in na execução padrão.
- Executados separadamente: enumeração de endpoints reais de áudio/energia e máscara Win32 em HWND oculto com pontos dentro, fora e nas lacunas; ambos passaram.
- TypeScript e Vite de produção passaram. O shader WebGL2 real foi renderizado em Edge headless com SwiftShader, usando textura ilustrativa: linhas refratadas, separação RGB, união a 16 px e texto fora do filtro. Evidência: `images/fusion-preview.png`. A primeira tentativa no sandbox falhou na inicialização GPU; a execução autorizada fora dele produziu a imagem.
- Não comprovados por essa cena: desempenho na GPU do usuário, arraste interativo no WebView2, DPI misto, fullscreen, recuperação do Explorer ou instalação/desinstalação. Não há alegação de 60 FPS nem equivalência visual exata a outro sistema.
- Com Wallpaper Engine, o modo unido usa transparência e reflexos. Refração RGB de vídeo ao vivo não está implementada; refração real exige wallpaper estático.

Verificação manual desta versão: ativar Widgets unidos; aproximar e afastar dois widgets com edição ligada; confirmar união e clique no desktop nas lacunas; cancelar arraste com Esc; reiniciar e conferir posições; desativar união e conferir posições individuais/notas. Testar Material → Fundo animado ligado/desligado, cor e dispersão. Abrir Control Center pela Top Bar, escolher endpoint, ajustar volume/mute, retirar o dispositivo, fechar por Esc/perda de foco e verificar os atalhos às configurações.
Build Rust release e instalador NSIS x64 0.7.0 concluídos com sucesso. Tamanho: 2431688 bytes. SHA-256: B4D0C53FB3BD438F69CCF633428FEAAC467127D7161A7A0D96F6A4F68F541CD2.

## 0.7.1 — correção de resolução e vidro em overlays

Passaram 28 testes JavaScript (incluindo mapas em DPI 2×), 15 testes Rust e, separadamente, o teste nativo da máscara com lacunas. TypeScript/Vite passaram. O teste de simetria foi atualizado para amostrar coordenadas proporcionais à resolução, preservando as verificações de direção, neutralidade e simetria.

Cena do shader real inspecionada em Edge headless/SwiftShader: `images/glass-071.png`. Inclui uma barra fina, notch com topo reto e widgets unidos, contornos antialiased e separação RGB. Isso valida compilação/renderização do shader, não a integração interativa no WebView2 nem uma comparação de desempenho na GPU do usuário.

Pendentes: inspeção no desktop do usuário da expansão da ilha, barra com/sem lacuna, DPI misto e comparação do serrilhado relatado. Não há captura ao vivo do Wallpaper Engine. Refração estática nas superfícies é opt-in; áreas acima de aplicativos amostram o wallpaper, não esses aplicativos.
Build release e NSIS 0.7.1 concluídos. Instalador x64: 2424867 bytes. SHA-256: 435E9C0740EC97963ACC471F53FA263A08CA7F7C24C0F98656B716460B3C61AD.

## 0.8.0 — Wallpaper Engine animado e material compartilhado

- Passaram 30 testes JavaScript e 18 testes Rust padrão. Incluem protocolo RGBA binário, frames truncados/grandes, coordenadas negativas, máscara de aplicativos e migração do tema sem perder notas/geometria. Doze testes nativos permanecem opt-in na suíte padrão.
- Teste nativo de exclusão/restauração de captura em HWND oculto: passou.
- Captura direta do renderer WPEVideoWallpaper e do host WorkerW: ambos rejeitados pelo Windows 10 22H2 (E_INVALIDARG). A implementação passou a usar WGC do monitor com máscara de aplicativos nessa situação.
- Teste controlado com um pequeno retângulo temporário sem ativação: observou duas cores sucessivas pelo broker real; passou. O retângulo foi removido por RAII. A exceção da máscara para o retângulo só existe em cfg(test), nunca no executável distribuído.
- Primeiro teste com desktop majoritariamente coberto: 39 frames entregues, 20.544 pixels de wallpaper visíveis e sem mudança. O usuário informou que estava vendo vídeo e solicitou nova captura.
- Nova captura com wallpaper reproduzindo: 7 frames, 1.181.716 pixels de wallpaper visíveis e diferença entre frames; passou. Nenhum frame do desktop foi salvo em disco. Esse teste confirma entrega/mudança dos frames, não FPS ou consumo durante uso prolongado.
- TypeScript/Vite e Rust release/NSIS passaram. A integração interativa final dentro do WebView2, gravações de terceiros, HDR, múltiplas GPUs e DPI misto continuam pendentes. Não há promessa de 60 FPS.

Roteiro: instalar após sair da versão anterior; verificar sincronização do material entre widgets, barra e ilha Liquid/Adaptive; manter Fundo animado ligado e Wallpaper Engine reproduzindo; testar dispersão/refração e aproximar widgets unidos. Cobrir uma região com outro aplicativo deve fazer o vidro dessa região usar translucidez. Alterar para fundo estático deve manter o alinhamento e remover exclusão de captura quando não houver outra superfície animada. Confirmar que notas e posições foram preservadas e que settings-before-live-glass.json foi criado na primeira gravação da migração.
Shader final inspecionado em cena ilustrativa no Edge/SwiftShader: images/glass-080.png. Instalador final: 2457376 bytes; SHA-256: 7FF5F579455EE33155561EB858A1FCD17634A75D64D19176B470D86657B01C47.

## 0.8.1 — regressões do fundo animado

- 32 testes JS passaram: composição equivalente para fundo capturado/mascarado, incluindo tintas vermelhas saturadas; proporções da lente em barras de 36 px.
- 19 testes Rust passaram; 12 testes dependentes do desktop ignorados na execução padrão. Rotação, máscara e geometria cobertas.
- Integração DXGI fora do sandbox passou: captura de duas cores controladas pelo broker real; janela sobreposta excluída da captura preservou o fundo. Janelas temporárias removidas por RAII.
- Wallpaper Engine real: 3 frames, 14.672 pixels visíveis, movimento detectado comparando RGB somente onde ambos os frames tinham alpha 255. Pixels ficaram em memória, sem imagens salvas. Grande parte do desktop estava coberta por aplicativos.
- Aparência final no desktop do usuário e ausência visual da borda após encerrar a versão antiga ainda exigem confirmação; os testes não substituem essa inspeção.
- WebGL2 no Edge/SwiftShader: shader compilou e a emenda capturado/mascarado ficou em 0,475/255. Reprodução: execute `node scripts/check-glass-webgl.mjs` e abra `.tools/check-glass.html`; o resultado esperado é PASS. Teste sintético; não é inspeção visual do desktop.

## 0.9.2 — notch sobre aplicativos

- A política de captura é por superfície: ilha e Top Bar preservam pixels de aplicativos; widgets continuam mascarando-os.
- Teste sintético com aplicativo escuro e placeholder branco: pixels escuros preservados na ilha, bloco branco tornado transparente, mesma área mascarada no widget.
- Sem acesso visual ao desktop nesta sessão; a aparência sobre o VS Code da foto requer conferência após instalar.

## 0.10.0 — coleção de widgets

- `npm run build` passou; `npm test`: 32/32; `cargo test --offline --manifest-path src-tauri/Cargo.toml`: 23/23 (13 testes opcionais ignorados). A suíte cobre compilação web, lógica compartilhada e migração das preferências.
- Leituras nativas opt-in passaram no Windows: GSMTC retornou duas sessões e uma faixa ativa após sair do sandbox; Core Audio leu volume e pico real; discos reportaram capacidade; CPU/RAM retornaram amostras reais. Dentro do sandbox, GSMTC retornou `0x80070005` (acesso negado).
- Media Player usa o provedor GSMTC já usado pela ilha. O medidor consulta `IAudioMeterInformation`; quando indisponível, o medidor fica vazio.
- Clima usa endpoints oficiais de previsão e geocodificação Open-Meteo. Configure cidade no painel; confirme retorno real no widget com rede disponível.
- Consulta direta fora do sandbox confirmou previsão de São Paulo com temperatura atual, 120 horários e 5 dias; a geocodificação retornou São Paulo/Brasil com coordenadas. A chamada dentro do sandbox não tinha acesso à rede.
- Galeria valida assinatura PNG, JPEG, WebP ou GIF, limita tamanho e guarda arquivos em `app_data_dir/gallery`; testa-se importando fotos e uma pasta pelo widget.
- Armazenamento consulta `sysinfo::Disks`; a seleção de unidade usa o ponto de montagem e pode mudar se o Windows remapear a unidade.
- Sem acesso visual confiável ao desktop durante o build, confira os nove widgets sobre wallpaper estático e Wallpaper Engine depois de instalar. Testes nativos opcionais que precisam do desktop permanecem ignorados na suíte padrão.
- Instalador NSIS 0.10.0 gerado: 2.521.319 bytes; SHA-256 `E5126D93D6D49EFE273FA4891BB1CC234231CF673EC0EE4E1CFFBAFCF73D12EA`.

## 0.10.1 — arraste e vidro dos novos widgets

- A lista de janelas em `capabilities/default.json` agora contém todos os nove IDs. Teste de regressão compara ACL e catálogo nativo.
- Media Player e Galeria não marcam mais suas superfícies inteiras como `data-no-drag`; controles continuam interativos.
- Inspeção do Z-order do desktop em execução: oito janelas visíveis de widget nas posições 418–425, acima dos `WorkerW` visíveis nas posições 426–427. A falha de permissão em `outer_position` impedia calcular a geometria da lente e é a causa provável da aparência incorreta.
- `npm run build` e 32 testes JS passaram; 24 testes Rust passaram, com 13 opcionais ignorados. Os testes opt-in de movimentação e ordenação de janelas ocultas também passaram no desktop real.
- Verificação visual após instalar 0.10.1 ainda é necessária para confirmar o resultado final sobre o Wallpaper Engine.
- Instalador NSIS 0.10.1: 2.534.410 bytes; SHA-256 `B334F91881715618438C90F836BBCB7D8AFD88BDD2631E7553B57F40D60964E8`.
