Primeira pré-release Windows x64 do AutoFish Local, com identidade visual própria
e captura restrita à janela do Roblox. Versão experimental para testes.

### Alterações

- Logo original removida do painel e do guia; novo ícone AF.
- Navegação horizontal, fundo verde petróleo e histórico em cartões.
- Windows Graphics Capture associado somente à janela validada do Roblox.
- Falha persistente de captura para a automação e libera o mouse.
- Painel não reaparece após fechamento manual; previews exibem falhas separadamente.

### Instalação e uso

Baixe `AutoFish-Local-v0.1.2-alpha-win-x64.zip`, extraia e execute
`AutoFish-Local.exe`. Requer Windows x64 e Microsoft WebView2. Não há instalação
automática ou atualizador. O aplicativo inicia parado.

Abra o GPO, equipe a vara, selecione a isca e calibre as regiões e o ponto de
lançamento. F1 inicia/pausa; F2 abre o editor; F3 encerra; F4 alterna o HUD.
**Ctrl+Alt+F12 é a parada de emergência fixa.**

### Validação

- Captura real positiva no GPO e detecção da barra em leituras consecutivas.
- Um ciclo automático completou o rastreamento; o resultado do peixe não foi
  confirmado pelo OCR e foi registrado como não verificado.
- 50 testes Rust aprovados e dois ignorados na correção de captura; Clippy aprovado.
- Build TypeScript/Vite, cinco verificações de segurança do frontend e inspeção
  visual da interface 0.1.2 aprovados.
- Microsoft Defender não encontrou ameaças no pacote local. Esse resultado não
  constitui certificação de segurança.

### Limitações conhecidas

- OCR do GPO pode ler texto de HUD alheio ao resultado; contadores não verificados
  não comprovam uma captura de peixe. Compatibilidade do OCR portátil em outros
  Windows permanece pendente.
- WebView2 gerou conexões HTTPS próprias na validação anterior, com webhooks
  desligados. O conteúdo não foi analisado; a distribuição não é certificada
  como estritamente offline.
- Fullscreen, outras GPUs/DPI/resoluções, pesca prolongada, compra e armazenamento
  ainda precisam de validação. Comece com funções auxiliares desligadas.
- Binário sem assinatura Authenticode comercial. Não desative proteções.

Os dados de configuração existentes são preservados. O ZIP não contém logs
pessoais, imagens capturadas, configuração de usuário ou credenciais. Webhooks
ficam desligados no padrão; qualquer URL configurada fica em texto claro nos
dados locais. As capturas de imagem não são enviadas pelo aplicativo.

Confira o SHA-256 do ZIP com o arquivo `.zip.sha256` anexado. `SHA256SUMS.txt`
dentro do ZIP contém os hashes dos arquivos distribuídos. Licença MIT original
preservada. Os relatórios de captura e interface acompanham o pacote.
