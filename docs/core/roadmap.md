[← Summary](./SUMMARY.md)

# Easy Cloud - Roadmap

Este documento centraliza a visão estratégica, as ideias, melhorias e potenciais novas funcionalidades para o futuro do projeto Easy Cloud.

## 🗺️ Visão Estratégica

O **Easy Cloud** nasce com o objetivo de ser uma **ferramenta de gestão de infraestrutura Cloud completa (All-in-one)**. O S3 é apenas o nosso ponto de partida. 

Nossa estratégia é oferecer uma suíte ampla (abrangendo múltiplos serviços como CloudFormation, EC2, IAM, Billing, etc.) para criar volume e dependência na ferramenta. Uma vez que o usuário esteja habituado ao ecossistema, afunilaremos e focaremos nossos esforços de monetização e recursos premium na gestão de **Storage (S3)**, onde temos o maior potencial de agregar valor com diferenciais únicos.

---

## 🏗️ Fase 1: Expansão Horizontal (Ecossistema Cloud)

- [ ] **Gestão de CloudFormation (Já iniciado)**
  - Criação, deploy e monitoramento de Stacks diretamente pelo app.
- [ ] **Integração com Billing & Cost Explorer**
  - Gráficos visuais mostrando gastos por serviço.
- [ ] **Suporte a Múltiplos Provedores**
  - Preparar terreno para conectar contas da Azure, GCP ou provedores S3-compatible (Cloudflare R2, MinIO, Wasabi).
- [ ] **Monitoramento Básico (CloudWatch / EC2)**
  - Listagem rápida de instâncias rodando e status de serviços principais.

---

## 🎯 Fase 2: O Funil do Storage (Monetização e Diferenciais)

Após consolidar a ferramenta, o grande apelo comercial (vender o peixe) será focado em revolucionar a forma como o usuário interage com o S3.

- [ ] **Criptografia Ponta a Ponta (E2E) - Zero Knowledge**
  - Adicionar criptografia client-side local via Rust antes do upload.
  - Troca de chaves via RSA para segurança e autenticidade.
  - Criptografia simétrica AES-256-GCM para performance nos arquivos pesados.
  - O provedor (AWS) não saberá o que há dentro do arquivo.
- [ ] **Sincronização Local (Estilo Dropbox/Google Drive)**
  - Assistente rodando em background (System Tray) fazendo sync bidirecional automático com buckets específicos.
- [ ] **Gestão Avançada de Custos de Storage**
  - Sugestões automáticas baseadas em IA para mover arquivos para Glacier/Deep Archive.
- [ ] **Compartilhamento Seguro Premium**
  - Links com senha e expiração criptografada pelo próprio app, voltado para equipes corporativas B2B.

---

## 🔮 Ideias Futuras

- [ ] **Integração com IA (LLMs)** para gerar templates do CloudFormation baseados em texto natural.
- [ ] **Terminal Embutido** com credenciais da AWS já exportadas automaticamente no ambiente.
