# Validação

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
