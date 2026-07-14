# Easy Cloud — AWS SDK Capabilities

Documento de referência com todas as operações disponíveis nas libs AWS SDK já integradas no projeto.

---

## aws-sdk-sts

Serviço de identidade e credenciais temporárias.

| Método | Descrição | Status |
|--------|-----------|--------|
| `get_caller_identity` | Retorna account ID, ARN e user ID | ✅ Implementado |
| `assume_role` | Assume uma role temporária (cross-account ou elevação de privilégio) | Disponível |
| `get_session_token` | Gera credenciais temporárias (útil com MFA) | Disponível |
| `get_access_key_info` | Identifica a conta dona de uma access key | Disponível |

---

## aws-sdk-s3

Serviço de armazenamento de objetos.

### Buckets

| Método | Descrição | Status |
|--------|-----------|--------|
| `list_buckets` | Lista todos os buckets da conta | ✅ Implementado |
| `create_bucket` | Cria um novo bucket | Disponível |
| `delete_bucket` | Remove um bucket (precisa estar vazio) | Disponível |
| `head_bucket` | Verifica se bucket existe e se há permissão de acesso | Disponível |
| `get_bucket_location` | Retorna a região onde o bucket está hospedado | Disponível |

### Objetos

| Método | Descrição | Status |
|--------|-----------|--------|
| `list_objects_v2` | Lista objetos dentro de um bucket (com paginação e prefixo) | Disponível |
| `get_object` | Faz download do conteúdo de um objeto | Disponível |
| `put_object` | Faz upload de um objeto para o bucket | Disponível |
| `delete_object` | Remove um objeto específico | Disponível |
| `delete_objects` | Remove múltiplos objetos em uma única requisição | Disponível |
| `copy_object` | Copia objeto entre buckets ou paths | Disponível |
| `head_object` | Retorna metadata de um objeto sem baixar o conteúdo | Disponível |

### Configurações de Bucket

| Método | Descrição | Status |
|--------|-----------|--------|
| `get_bucket_versioning` | Verifica se versionamento está ativo | Disponível |
| `put_bucket_versioning` | Ativa ou desativa versionamento | Disponível |
| `get_bucket_policy` | Retorna a bucket policy (JSON) | Disponível |
| `put_bucket_policy` | Define ou atualiza a bucket policy | Disponível |
| `get_bucket_tagging` | Lista as tags do bucket | Disponível |
| `put_bucket_tagging` | Define tags no bucket | Disponível |
| `get_bucket_encryption` | Verifica configuração de server-side encryption | Disponível |

### Upload Avançado

| Método | Descrição | Status |
|--------|-----------|--------|
| `create_multipart_upload` | Inicia upload multipart para arquivos grandes | Disponível |
| `upload_part` | Envia uma parte do upload multipart | Disponível |
| `complete_multipart_upload` | Finaliza o upload multipart | Disponível |
| `abort_multipart_upload` | Cancela um upload multipart em andamento | Disponível |

### Acesso Temporário

| Método | Descrição | Status |
|--------|-----------|--------|
| `presigned URL` (via presigning config) | Gera URL temporária para acesso direto a um objeto | Disponível |

---

## aws-sdk-cloudformation

Serviço de infraestrutura como código.

### Stacks

| Método | Descrição | Status |
|--------|-----------|--------|
| `describe_stacks` | Lista e detalha stacks existentes | ✅ Implementado |
| `create_stack` | Cria uma nova stack a partir de um template | ✅ Implementado |
| `update_stack` | Atualiza uma stack existente com novo template | Disponível |
| `delete_stack` | Remove uma stack e todos seus recursos | Disponível |
| `describe_stack_events` | Retorna histórico de eventos (progresso de deploy) | Disponível |
| `describe_stack_resources` | Lista todos os recursos provisionados pela stack | Disponível |
| `list_stack_resources` | Lista recursos com paginação | Disponível |
| `describe_stack_resource` | Detalha um recurso específico de uma stack | Disponível |

### Templates

| Método | Descrição | Status |
|--------|-----------|--------|
| `get_template` | Retorna o template atual de uma stack já deployada | Disponível |
| `validate_template` | Valida sintaxe e estrutura de um template antes do deploy | Disponível |
| `estimate_template_cost` | Gera link para estimativa de custo baseado no template | Disponível |

### Change Sets

| Método | Descrição | Status |
|--------|-----------|--------|
| `create_change_set` | Cria um change set (preview de alterações sem aplicar) | Disponível |
| `describe_change_set` | Detalha as mudanças propostas no change set | Disponível |
| `execute_change_set` | Aplica as mudanças do change set na stack | Disponível |
| `delete_change_set` | Descarta um change set sem aplicar | Disponível |

### Drift Detection

| Método | Descrição | Status |
|--------|-----------|--------|
| `detect_stack_drift` | Inicia detecção de drift (mudanças manuais fora do CF) | Disponível |
| `describe_stack_drift_detection_status` | Verifica status da detecção de drift | Disponível |
| `describe_stack_resource_drifts` | Lista quais recursos estão em drift | Disponível |

---

## Funcionalidades Sugeridas

Com base nas operações acima, features que podem ser construídas sem dependências adicionais:

### 1. S3 File Browser
- Navegar dentro de buckets com `list_objects_v2`
- Upload/download de arquivos com `put_object`/`get_object`
- Deletar objetos com `delete_object`/`delete_objects`
- Criar e deletar buckets
- Visualizar metadata de objetos com `head_object`

### 2. CloudFormation Live Deploy
- Acompanhar deploy em tempo real com polling de `describe_stack_events`
- Mostrar progresso recurso a recurso
- Indicar sucesso/falha de cada evento

### 3. Stack Diff / Change Sets
- Preview de mudanças antes de aplicar (`create_change_set` + `describe_change_set`)
- Diff visual entre template atual e novo
- Aplicar ou descartar (`execute_change_set` / `delete_change_set`)

### 4. Template Validator
- Validar template direto no editor com `validate_template`
- Feedback inline de erros de sintaxe/estrutura
- Estimativa de custo com `estimate_template_cost`

### 5. Stack Drift Detection
- Detectar se alguém modificou recursos manualmente
- Listar recursos em drift com detalhes do que mudou
- Alertas visuais no painel de stacks

### 6. Cross-Account Access
- Assumir roles em outras contas via `assume_role`
- Gerenciar múltiplas contas com credenciais temporárias
- Suporte a MFA com `get_session_token`

### 7. Stack Management Completo
- Update de stacks existentes
- Delete de stacks com confirmação
- Visualizar template atual de qualquer stack (`get_template`)
- Histórico de eventos por stack

---

## Referências

- [AWS SDK for Rust Docs](https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/)
- [AWS CloudFormation API Reference](https://docs.aws.amazon.com/AWSCloudFormation/latest/APIReference/)
- [AWS STS API Reference](https://docs.aws.amazon.com/STS/latest/APIReference/)
