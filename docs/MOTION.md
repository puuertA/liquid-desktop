# Movimento e continuidade

## Implementado na 0.4.0

O motor usa uma mola amortecida analítica, com frequência 19 rad/s e razão de amortecimento 0,82. Cada superfície mantém posição, velocidade e destino. Retargeting conserva velocidade e posição; não reinicia o gesto do começo.

Pressão altera escala X para 1,004 e Y para 0,99. Soltar, sair, cancelar ponteiro, perder foco ou desmontar limpa o estado correspondente. GSAP agenda a atualização somente enquanto há deslocamento. Ao atingir tolerâncias de posição 0,0001 e velocidade 0,001, remove o ticker da superfície. Nenhuma animação decorativa em repouso. Controles de texto e botões não deformam o widget inteiro; arraste mantém a captura já validada.

`prefers-reduced-motion` e a configuração local podem desligar a resposta. Mudança da preferência durante a execução e desmontagem removem o ticker. O mapa óptico não depende do cursor: movimento do cursor atualiza apenas a iluminação, sem gerar PNGs. Ajustes de refração/dispersão alteram atributos do filtro, não o mapa.

## Contrato da futura Dynamic Island

Expansão deve começar horizontalmente e continuar em altura, mantendo um único contorno. Alterações de raio e região nativa precisam corresponder à forma visível. Ao interromper hover, nova trajetória parte da geometria e velocidade atuais. Conteúdo entra após espaço suficiente; a capa permanece o mesmo elemento entre compact e expanded. Movimento reduzido usa mudança direta de geometria e mantém todas as ações.

Fusão entre cápsulas deve existir apenas para atividades reais concorrentes. Não aplicar blur/goo ao texto ou aos ícones; limitar a operação à camada de material e à região de hit testing. A fusão ainda não está implementada. Nenhuma janela Dynamic Island ou Top Bar é entregue na 0.4.0.
