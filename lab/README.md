# Laboratório Docker — Cysee MVP ([S1] 2 Unidades + Central)

Este diretório contém o ambiente reproduzível em Docker Compose que substitui a topologia inicial do EVE-NG para validação do MVP do **Cysee**. O laboratório emula duas unidades monitoradas isoladas (Unidade A e Unidade B), um Núcleo Central e a camada de Gerência (Grafana).

---

## 1. Arquitetura dos Containers e Estrutura de Diretórios

```text
lab/
├── compose.yaml            # Orquestração de serviços, redes isoladas e volumes
├── README.md               # Documentação de topologia, operação e testes de aceite
├── snmp-emitter/
│   └── Dockerfile          # Imagem Alpine com net-snmp (snmpd, snmptrap, snmpinform)
├── cysee-agent/
│   ├── Dockerfile          # Multi-stage build oficial Rust -> Debian Bookworm Slim
│   ├── Cargo.toml
│   └── src/main.rs         # Binário Rust do Agente (UDP 162 + Outbox local + Sender)
└── cysee-central/
    ├── Dockerfile          # Multi-stage build oficial Rust -> Debian Bookworm Slim
    ├── Cargo.toml
    └── src/main.rs         # Binário Rust do Central (Ingest TCP 50051 + API HTTP 8080)
```

* **Containers Rust (`cysee-agent` e `cysee-central`)**: Construídos seguindo o padrão de *Multi-Stage Build* da documentação oficial do Docker para Rust. O estágio `builder` (`rust:1.82-slim-bookworm`) compila o binário com `cargo build --release`, e o estágio `runtime` (`debian:bookworm-slim`) copia apenas o executável final para `/usr/local/bin/`, mantendo a imagem leve e aderente ao ADR-0001 (binário único sem dependências de compilação).
* **Container Grafana (`grafana/grafana-oss`)**: Configurado conforme a documentação oficial do Grafana para Docker, utilizando persistência em volume nomeado (`grafana-storage:/var/lib/grafana`) e provisionamento automático do plugin **Infinity** (`GF_INSTALL_PLUGINS=yesoreyeram-infinity-datasource`) para leitura da API REST do Central.
* **Containers Emissores SNMP (`snmp-emitter-a` e `snmp-emitter-b`)**: Baseados em Alpine Linux com a suíte `net-snmp` (`snmpd`, `snmptrap` e `snmpinform`) para geração de tráfego SNMP v2c de teste.

---

## 2. Topologia de Redes, Endereçamento e Portas

Para garantir que a **Unidade A** e a **Unidade B** não possuam comunicação lateral, mas ambas alcancem o serviço de ingestão do **Central**, o ambiente utiliza 5 redes Docker distintas. O container `cysee-central` opera com `net.ipv4.ip_forward=0` via `sysctl`, impedindo que atue como roteador entre as sub-redes de ingestão.

### 2.1. Tabela de Redes Docker

| Rede | Sub-rede (CIDR) | Flag `internal` | Finalidade |
| :--- | :--- | :--- | :--- |
| `net_unit_a` | `172.20.10.0/24` | `true` | Segmento local da Unidade A (Emissor SNMP A <-> Agente A). |
| `net_unit_b` | `172.20.20.0/24` | `true` | Segmento local da Unidade B (Emissor SNMP B <-> Agente B). |
| `net_ingest_a` | `172.20.101.0/24` | `true` | Canal exclusivo de ingestão entre Agente A e Central. |
| `net_ingest_b` | `172.20.102.0/24` | `true` | Canal exclusivo de ingestão entre Agente B e Central. |
| `net_mgmt` | `172.20.99.0/24` | `false` | Rede de Gerência (Grafana <-> API REST do Central e acesso ao Host). |

### 2.2. Tabela de Serviços, IPs, Portas e Volumes

| Serviço / Container | Redes Conectadas | Endereços IP | Portas Internas / Host | Volume de Persistência |
| :--- | :--- | :--- | :--- | :--- |
| `snmp-emitter-a` | `net_unit_a` | `172.20.10.10` | `161/udp` (`snmpd`) | — |
| `cysee-agent-a` | `net_unit_a`<br>`net_ingest_a` | `172.20.10.2`<br>`172.20.101.2` | `162/udp` (Recepção Trap/INFORM) | `vol_agent_a:/var/lib/cysee` |
| `snmp-emitter-b` | `net_unit_b` | `172.20.20.10` | `161/udp` (`snmpd`) | — |
| `cysee-agent-b` | `net_unit_b`<br>`net_ingest_b` | `172.20.20.2`<br>`172.20.102.2` | `162/udp` (Recepção Trap/INFORM) | `vol_agent_b:/var/lib/cysee` |
| `cysee-central` | `net_ingest_a`<br>`net_ingest_b`<br>`net_mgmt` | `172.20.101.100`<br>`172.20.102.100`<br>`172.20.99.100` | `50051/tcp` (Ingest Gateway)<br>`8080/tcp` -> `localhost:8080` (API) | `vol_central:/var/lib/cysee` |
| `grafana` | `net_mgmt` | `172.20.99.10` | `3000/tcp` -> `localhost:3000` (Web) | `grafana-storage:/var/lib/grafana` |

---

## 3. Ciclo de Vida do Laboratório (Comandos de Operação)

Execute os comandos abaixo a partir do diretório `lab/`:

### Subir o ambiente (compilando as imagens Rust)
```bash
docker compose up -d --build
```

### Verificar o status dos containers e acompanhar os logs
```bash
docker compose ps
docker compose logs -f cysee-agent-a cysee-agent-b cysee-central
```

### Parar o laboratório preservando o estado local (outbox e central)
```bash
docker compose stop
```

### Destruir e recriar o laboratório sem estado residual
O flag `-v` remove os volumes nomeados (`vol_agent_a`, `vol_agent_b`, `vol_central` e `grafana-storage`), garantindo que nenhum dado de testes anteriores permaneça no ambiente:
```bash
docker compose down -v --remove-orphans
docker compose up -d --build
```

---

## 4. Roteiro de Validação dos Critérios de Aceite

### Critério 1: Subida e recriação limpa do ambiente
```bash
docker compose down -v --remove-orphans && docker compose up -d --build
docker compose ps
```
*Resultado esperado*: Todos os 6 containers (`snmp-emitter-a`, `cysee-agent-a`, `snmp-emitter-b`, `cysee-agent-b`, `cysee-central`, `grafana`) com status `Up` / `running`.

---

### Critério 2: Recepção de Trap SNMP v2c na UDP 162 (Unidade A e Unidade B)

1. Emita um trap SNMP v2c (`linkDown`) do **Emissor A** para o **Agente A** (`172.20.10.2:162`):
```bash
docker exec snmp-emitter-a snmptrap -v 2c -c public 172.20.10.2:162 '' \
  1.3.6.1.6.3.1.1.5.3 1.3.6.1.2.1.2.2.1.1.1 i 1
```

2. Emita um trap SNMP v2c (`linkDown`) do **Emissor B** para o **Agente B** (`172.20.20.2:162`):
```bash
docker exec snmp-emitter-b snmptrap -v 2c -c public 172.20.20.2:162 '' \
  1.3.6.1.6.3.1.1.5.3 1.3.6.1.2.1.2.2.1.1.2 i 2
```

3. *(Opcional)* Teste também o envio de `snmpinform` v2c (nota: enquanto o decodificador BER/ASN.1 completo da Semana 2 não estiver integrado no binário Rust para devolver um `Response-PDU`, o comando `snmpinform` acusará timeout no cliente após entregar os datagramas UDP na porta 162 do agente):
```bash
docker exec snmp-emitter-a snmpinform -v 2c -c public -t 1 -r 1 172.20.10.2:162 '' \
  1.3.6.1.6.3.1.1.5.4 1.3.6.1.2.1.2.2.1.1.1 i 1 || true
```

4. Verifique nos logs e nos volumes de persistência que os pacotes chegaram na UDP 162 de cada agente e foram confirmados (`ACK`) pelo Central:
```bash
# Eventos confirmados no Agente A e Agente B
docker exec cysee-agent-a cat /var/lib/cysee/outbox_acked.log
docker exec cysee-agent-b cat /var/lib/cysee/outbox_acked.log

# Eventos consolidados na API REST do Central
curl -s http://localhost:8080
```

---

### Critério 3: Isolamento entre Unidades e Conectividade com o Central

1. **Isolamento lateral (Unidade A -> Unidade B)**: Ambos os comandos abaixo **devem falhar** (`100% packet loss` / `Network is unreachable`), provando que a Unidade A não alcança a Unidade B nem pela LAN nem pela rede de ingestão:
```bash
docker exec cysee-agent-a ping -c 2 -W 2 172.20.20.2 || echo "OK: LAN da Unidade B isolada"
docker exec cysee-agent-a ping -c 2 -W 2 172.20.102.2 || echo "OK: Uplink da Unidade B isolado"
```

2. **Isolamento dos dispositivos SNMP**: O emissor SNMP da Unidade A não deve alcançar o Central diretamente (**deve falhar**):
```bash
docker exec snmp-emitter-a ping -c 2 -W 2 172.20.101.100 || echo "OK: Emissor isolado do Central"
```

3. **Acesso das unidades ao Ingest Gateway do Central**: Ambos os agentes devem conseguir alcançar a porta `50051/tcp` do Central (**deve ter sucesso**):
```bash
docker exec cysee-agent-a nc -zv 172.20.101.100 50051
docker exec cysee-agent-b nc -zv 172.20.102.100 50051
```

---

### Critério 4: Bloqueio Temporário (Partição de Rede), Acumulação Local e Retomada

1. Bloqueie o tráfego de saída do **Agente A** para a porta de ingestão do Central (`50051`) usando `iptables`:
```bash
docker exec cysee-agent-a iptables -A OUTPUT -d 172.20.101.100 -p tcp --dport 50051 -j DROP
```

2. Envie 3 traps SNMP v2c a partir do **Emissor A** enquanto o Agente A está sem comunicação com o Central:
```bash
for i in 1 2 3; do
  docker exec snmp-emitter-a snmptrap -v 2c -c public 172.20.10.2:162 '' \
    1.3.6.1.6.3.1.1.5.3 1.3.6.1.2.1.2.2.1.1.1 i $i
done
```

3. Verifique que os 3 eventos estão acumulados localmente no arquivo de pendentes (`outbox_pending.log`) do Agente A:
```bash
docker exec cysee-agent-a cat /var/lib/cysee/outbox_pending.log
```

4. Remova a regra de bloqueio no `iptables` para restabelecer a comunicação com o Central:
```bash
docker exec cysee-agent-a iptables -D OUTPUT -d 172.20.101.100 -p tcp --dport 50051 -j DROP
```

5. Aguarde 2 segundos e verifique que a fila pendente (`outbox_pending.log`) esvaziou, os eventos migraram para confirmados (`outbox_acked.log`) e chegaram ao Central:
```bash
sleep 2
docker exec cysee-agent-a cat /var/lib/cysee/outbox_pending.log
docker exec cysee-agent-a cat /var/lib/cysee/outbox_acked.log
curl -s http://localhost:8080
```

---

## 5. Acesso ao Grafana (Rede de Gerência)

1. Acesse `http://localhost:3000` no navegador (usuário padrão `admin` / senha `admin`).
2. Em **Connections -> Data sources -> Add new data source**, selecione o plugin **Infinity** (já pré-instalado no container).
3. Para consultar o Central pela rede interna de gerência (`net_mgmt`), utilize a URL `http://172.20.99.100:8080` na criação do painel.

---

## 6. Limites do Laboratório

* As redes `bridge` do Docker isolam a conectividade IP (Camada 3) entre os containers das unidades, mas não substituem switches físicos nem validam marcação de VLANs IEEE 802.1Q (Camada 2) ou o comportamento de hardware SNMP real (como limites de buffer em ASICs, MIBs proprietárias ou desvios de implementação em roteadores/switches físicos).
* Os binários em `cysee-agent/` e `cysee-central/` presentes nesta task inicial funcionam como *stubs* em Rust para validação da infraestrutura de rede, portas, persistência em volume e fluxo de ACK; a decodificação BER/ASN.1, o banco SQLite em modo WAL (`outbox.db` / `central.db`) e o transporte gRPC/TLS serão integrados nas entregas subsequentes do cronograma do MVP.