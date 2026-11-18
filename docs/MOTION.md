# Movimento e continuidade

## Dynamic Island — 0.5.1

Geometrias lógicas: Idle 140×36/r18, Compact 320×52/r26, Multi 390×56/r28, Notification 340×62/r28, Expanded 440×306/r32. Mola analítica preserva velocidade em interrupções: largura/raio 18 rad/s, amortecimento 0,9; altura 15,5 rad/s, amortecimento 0,92. A altura começa junto, mas progride mais lentamente; não existe pausa artificial. Capa e cabeçalho interpolam pela altura atual. O conteúdo interativo entra após 130 ms com fade/translate e inert enquanto oculto. Retirar o cursor usa tolerância de 180 ms. Esc recolhe; perda de foco respeita fixação explícita.

Região nativa recebe a geometria alterada por frame, quantizada pelo DPI, com envio final obrigatório. Uma fila única persiste entre transições, mantém apenas o último frame pendente e um envio em andamento. Isso remove o limite artificial de 30 Hz; não garante FPS real do compositor. CSS e HWND compartilham largura/altura/raio. Movimento reduzido aplica geometria diretamente. A capa permanece no mesmo elemento. Multi-Activity apresenta mídia e chip de timer no mesmo contorno; não implementa metaballs.

Iluminação da ilha e dos widgets usa interpolação exponencial independente da taxa de frames (14/s). Não gera mapas ópticos nem renderizações React por movimento. O ticker termina ao convergir e é removido no blur/desmonte; movimento reduzido fixa a luz.

## Implementado na 0.4.0

O motor usa uma mola amortecida analítica, com frequência 19 rad/s e razão de amortecimento 0,82. Cada superfície mantém posição, velocidade e destino. Retargeting conserva velocidade e posição; não reinicia o gesto do começo.

Pressão altera escala X para 1,004 e Y para 0,99. Soltar, sair, cancelar ponteiro, perder foco ou desmontar limpa o estado correspondente. GSAP agenda a atualização somente enquanto há deslocamento. Ao atingir tolerâncias de posição 0,0001 e velocidade 0,001, remove o ticker da superfície. Nenhuma animação decorativa em repouso. Controles de texto e botões não deformam o widget inteiro; arraste mantém a captura já validada.

`prefers-reduced-motion` e a configuração local podem desligar a resposta. Mudança da preferência durante a execução e desmontagem removem o ticker. O mapa óptico não depende do cursor: movimento do cursor atualiza apenas a iluminação, sem gerar PNGs. Ajustes de refração/dispersão alteram atributos do filtro, não o mapa.

## Contrato da futura Dynamic Island

Expansão deve começar horizontalmente e continuar em altura, mantendo um único contorno. Alterações de raio e região nativa precisam corresponder à forma visível. Ao interromper hover, nova trajetória parte da geometria e velocidade atuais. Conteúdo entra após espaço suficiente; a capa permanece o mesmo elemento entre compact e expanded. Movimento reduzido usa mudança direta de geometria e mantém todas as ações.

Fusão entre cápsulas deve existir apenas para atividades reais concorrentes. Não aplicar blur/goo ao texto ou aos ícones; limitar a operação à camada de material e à região de hit testing. A fusão ainda não está implementada. A primeira janela Dynamic Island foi adicionada na 0.5.0; a Top Bar 0.6.0 mantém geometria estável e não executa animação decorativa em repouso.
