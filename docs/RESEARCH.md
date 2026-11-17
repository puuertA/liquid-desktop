# Pesquisa técnica e licenças

Consultada em 16/11/2026, antes da implementação.

- [Tauri: configuração](https://v2.tauri.app/reference/config/): transparência/decoração são propriedades de janela. Não fornecem por si sós integração com o desktop nem hit-test por alfa.
- [Microsoft: IDesktopWallpaper](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idesktopwallpaper): contrato público para imagem por monitor, retângulo, posicionamento e cor. É a fonte escolhida para refração estática.
- [Microsoft: SetParent](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent): requer ajustar WS_CHILD/WS_POPUP; reparenting entre processos com DPI diferente pode resetar awareness. Portanto, múltiplos DPI não podem ser declarados validados apenas porque coordenadas são físicas.
- [Microsoft: SetWindowRgn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowrgn): recorta a superfície nativa. Usado para não deixar cantos transparentes retangulares capturando mouse.
- [Lively](https://github.com/rocksdanister/lively), [licença GPL-3.0](https://github.com/rocksdanister/lively/blob/core-separation/LICENSE): referência de projeto para hospedagem de wallpaper no Explorer. Nenhum código copiado, para não importar obrigações GPL inadvertidamente.
- [WallpaperPlatform](https://github.com/eaengle/WallpaperPlatform): referência investigada para diferenças de topologia WorkerW entre versões do Explorer. Não foi incorporado código nem dependência deste projeto.

## Decisão provisória

**Histórico das versões até 0.1.1, substituído em 0.1.2.** O erro real relatado pelo usuário mostrou que o widget podia estar sem pai válido; os testes antigos com um STATIC filho não cobriam o ciclo de vida do WebView2/Tauri.

WorkerW atrás de DefView é apropriado para wallpapers; não resolve automaticamente input de widgets, pois a camada de ícones ocupa o desktop. Não enviar mouse sintético nem alterar o Explorer para contornar isso nesta prova. Um child HWND pequeno no host de ícones Progman/WorkerW é uma hipótese testável para interação. Recusar topologias não reconhecidas. Não afirmar suporte geral antes de testes locais.

SVG não amostra pixels de outras janelas. `backdrop-filter` do WebView2 não é uma lente sobre o Windows. Usar imagem real do wallpaper em canvas e distorção via feDisplacementMap. Isso não refrata ícones, vídeos nem janelas externas. WebView2 pode apresentar diferenças de filtros, composição e flashes na abertura; verificar visualmente no runtime final.

O código original deste projeto usa MIT. Dependências mantêm suas respectivas licenças; revisar inventário e notices antes de distribuição pública.

## Bandeja — 0.3.0

Bandeja e menu seguem a API oficial [System Tray do Tauri](https://v2.tauri.app/learn/system-tray/). O registro de inicialização usa windows-registry em HKCU explicitamente, evitando o modo Dynamic da dependência auto-launch, que tenta configuração para todos os usuários quando o processo está elevado. Nenhuma escrita no Registro é feita durante a compilação.

## Arraste direto

A versão 0.1.1 usa captura de ponteiro do WebView para dirigir os comandos de início, movimento e soltura. [GetPhysicalCursorPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getphysicalcursorpos) fornece coordenadas físicas na thread principal, e SetWindowPos move apenas o HWND do widget. A versão anterior consultava [GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate) numa tarefa de fundo; esse caminho foi removido por depender de estado global de entrada em vez do gesto recebido pelo WebView. Não foi possível comprovar que essa dependência era a causa única do problema relatado.

## Janela independente — 0.1.2

A mensagem enviada pelo usuário localizou a falha na exigência de IsWindow(GetParent(hwnd)), inválida para top-level sem owner. Movimentação e tamanho passaram a usar diretamente coordenadas de tela. [SetWindowPos](https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setwindowpos) documenta a ordem e os estados topmost; o código usa uma janela não-topmost e reposiciona somente o próprio widget acima da superfície do shell. Refração estática foi desativada por padrão para não simular vídeo do Wallpaper Engine com uma imagem antiga. Nenhum código do Rainmeter foi copiado.

## Faixa persistente — 0.1.4

A captura do usuário demonstrou que limpar os bits de estilo não bastava. Conforme [WM_NCCALCSIZE](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-nccalcsize), retornar zero sem alterar os retângulos faz o conteúdo ocupar a janela inteira. A implementação intercepta também WM_NCPAINT e preserva a lógica de foco de Tao via [WM_NCACTIVATE com lParam=-1](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-ncactivate). [SetWindowSubclass](https://learn.microsoft.com/en-us/windows/win32/api/commctrl/nf-commctrl-setwindowsubclass) exige a thread proprietária: a configuração nativa foi movida para run_on_main_thread. Testes verificam origem e tamanho do client rect, não somente flags.
