# Liquid Desktop — linguagem visual

A superfície deve parecer uma lente sobre o wallpaper real. Transparência e blur, sozinhos, não atendem ao objetivo.

## Material

Na versão 0.2.0, o material é compartilhado pelos cinco tipos de widget. O mapa de deslocamento acompanha a largura e altura configuradas. Cada janela conserva sua cor, opacidade, reflexos e contraste; textos e controles permanecem sobre a camada óptica. O painel separa coleção e configurações do widget selecionado. Notas e botões do calendário não iniciam arraste. Gráficos do monitor exibem somente amostras reais recebidas, sem curvas decorativas ou valores de demonstração.

Relógio de 360 × 220 unidades lógicas; raio 32. Mapa normal RGB: vermelho desloca X, verde desloca Y. A distorção se concentra nos 26 px próximos ao contorno arredondado, com queda quadrática para preservar a leitura central. Deslocamento padrão 18 px, ajustável 0–40. Véu claro padrão 12%, máximo 60%. Blur inicial 0,35 px. Reflexos discretos no contorno, luz radial acompanhando o cursor.

A textura é derivada exclusivamente do wallpaper atual. Nunca inserir wallpaper decorativo de substituição no widget. Erros devem ser visíveis, sem dados simulados. O desenho abstrato do painel representa material e não é uma prévia do desktop.

## Composição

Segoe UI Variable com fallback Segoe UI. Relógio 68 px, peso 450, numerais tabulares. Painel em tons minerais claros e verdes dessaturados, espaços 8/12/16/24/32. Hierarquia curta: título, ação, material, posição, diagnóstico. A camada de legibilidade rgba(10,18,24,.62) fica acima da textura/reflexos e abaixo do conteúdo branco. Isso preserva contraste mesmo sobre branco; labels usam 14 px e peso 600. Avisos têm área própria, 12 px, sem posição absoluta sobre instruções. Adaptação por luminância poderá substituir a camada fixa após validação.

## Movimento

Pressão: escala 0,985 em 160 ms. Retorno elástico em 550 ms, amplitude contida. O texto não deve perder legibilidade. Respeitar `prefers-reduced-motion`; não usar loops decorativos. Suspender desenho quando o documento estiver oculto. Futuras animações de dock e morphing só entram após medir custo.

## Proibições

Sem neon, brilho intenso, gradiente em texto, sombras pesadas, fundo falso de macOS, métricas inventadas ou transparência que bloqueie o desktop fora da superfície. Não afirmar refração de conteúdo que não esteja realmente disponível como textura.

## Ajustes 0.1.3

Cor do material configurável (#RRGGBB), reflexos 0–100%, contraste 20–75%. A camada fixa de 62% foi substituída por uma camada radial ajustável (padrão 42%); a razão de contraste histórica não se aplica a todos os ajustes. Texto branco mantém sombra curta para separar o conteúdo do fundo. A borda usa um anel mascarado de 1,5 px, um reflexo interno a 5 px do contorno e um brilho inferior tingido pela cor escolhida. A iluminação segue o cursor, sem animação contínua em repouso. No modo desktop não mostrar instruções de edição. Refração de vídeo continua fora do escopo.
