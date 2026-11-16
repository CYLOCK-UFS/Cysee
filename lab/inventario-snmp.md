# Inventário de capacidades SNMP — laboratório Cysee

**Escopo: SNMP v2c. Estado: parcial / bloqueado para inventário de equipamentos reais.** O repositório descreve dois segmentos Docker, mas não identifica nem conecta equipamentos físicos/virtuais com CLI `show`. Os únicos dispositivos SNMP configurados são emissores `net-snmp` em containers. Configuração versionada serve como evidência do laboratório emulado, não como evidência de `show version` ou de capacidades de hardware. v1/v3 só precisam ser investigados se um equipamento não puder operar em v2c e isso bloquear o MVP. Não houve captura de pacote nesta execução; nenhum `.pcap` foi fabricado.

## Inventário por segmento

| Segmento / dispositivo | Modelo / firmware (evidência) | SNMP v1 / v2c / v3 | INFORM | Community | Traps / varbinds emitidos | Evidência disponível |
|---|---|---|---|---|---|---|
| Unidade A — `snmp-emitter-a` (`172.20.10.10`) | Dispositivo emulado; imagem `alpine:3.20`. Modelo/firmware de equipamento: não aplicável. | v2c configurado (`rocommunity public`); verificar operação v2c no teste. v1/v3 fora da validação, salvo incompatibilidade que bloqueie o MVP. | Não verificado. | `public` RO — `lab/snmp-emitter/Dockerfile`, `rocommunity public`. | O roteiro especifica `linkDown` (`1.3.6.1.6.3.1.1.5.3`) com `ifIndex` (`1.3.6.1.2.1.2.2.1.1.1`, INTEGER); envio/captura não verificados. Outros tipos e ordem de varbinds não capturados. | `lab/compose.yalm` (IP/rede) e `lab/snmp-emitter/Dockerfile` (imagem/config); `lab/README.md`, critério 2 (comando proposto, não resultado observado). |
| Unidade B — `snmp-emitter-b` (`172.20.20.10`) | Dispositivo emulado; imagem `alpine:3.20`. Modelo/firmware de equipamento: não aplicável. | v2c configurado (`rocommunity public`); verificar operação v2c no teste. v1/v3 fora da validação, salvo incompatibilidade que bloqueie o MVP. | Não verificado. | `public` RO — `lab/snmp-emitter/Dockerfile`, `rocommunity public`. | O roteiro especifica `linkDown` (`1.3.6.1.6.3.1.1.5.3`) com `ifIndex` (`1.3.6.1.2.1.2.2.1.1.1`, INTEGER); envio/captura não verificados. Outros tipos e ordem de varbinds não capturados. | `lab/compose.yalm` (IP/rede) e `lab/snmp-emitter/Dockerfile` (imagem/config); `lab/README.md`, critério 2 (comando proposto, não resultado observado). |

### Evidência de configuração disponível no repositório

```text
# lab/snmp-emitter/Dockerfile
FROM alpine:3.20
RUN apk add --no-cache net-snmp net-snmp-tools iputils-ping iproute2 netcat-openbsd
RUN printf "rocommunity public\nsysLocation Lab_Cylock_UFS\n" > /etc/snmp/snmpd.conf
```

A definição de rede/IP está em `lab/compose.yalm`; os dois containers usam a mesma imagem/configuração. Não há saída de `show version`, `show run`, `snmpwalk` ou captura bruta arquivada no repositório. O OID `1.3.6.1.6.3.11.2.1.1.0` (a coluna do grupo `snmpTargetParams` não é, por si só, uma prova de suporte a INFORM) também não foi consultado. Para dispositivos reais, registrar a saída dos comandos e a documentação do modelo antes de marcar qualquer capacidade como confirmada.

## Traps previstos no roteiro, não confirmados por captura

| Nome | OID | Varbinds na ordem do comando proposto |
|---|---|---|
| `linkDown` | `1.3.6.1.6.3.1.1.5.3` | `1.3.6.1.2.1.2.2.1.1.1` (`ifIndex`, INTEGER; valor 1 na Unidade A, 2 na B). |
| `linkUp` | `1.3.6.1.6.3.1.1.5.4` | Nenhum varbind adicional especificado no comando de INFORM do roteiro. |
| `coldStart`, `warmStart`, `authenticationFailure` | `1.3.6.1.6.3.1.1.5.1`, `.2`, `.5` | Não emitidos/capturados. |

A ordem acima é a dos argumentos nos comandos de `lab/README.md`, não uma observação dos bytes enviados. Confirmar cada tipo, incluindo os varbinds padrão SNMPv2 (`sysUpTime.0` e `snmpTrapOID.0`) e a ordem efetivamente codificada, lendo uma captura com `snmptrapd -f -Lo -d` ou `tcpdump`/`tshark`.

## Como completar a evidência e capturas

1. Subir o laboratório e confirmar os dois emissores e agentes ativos (`docker compose -f lab/compose.yalm up -d --build`; `docker compose -f lab/compose.yalm ps`). Para equipamento real, executar e salvar a saída de `show version`, `show run | include snmp`, `show run | include community` e, se suportado, `snmpwalk -v2c -c <community> <ip> 1.3.6.1.6.3.11.2.1.1.0`.
2. No emissor, verificar a configuração observada com `docker exec snmp-emitter-a cat /etc/snmp/snmpd.conf` (repetir para B). Isso verifica apenas o emulador, não equipamento real.
3. Capturar na interface/rede do segmento enquanto se envia o trap do roteiro: `tcpdump -ni any -s0 -w lab/pcap/unidade-a-linkdown.pcap 'udp port 162'`. Repetir para cada tipo e unidade; assegurar que o arquivo não esteja vazio e decodificar para documentar OID e ordem dos varbinds. Criar `lab/pcap/` somente quando houver captura real.

## Configuração e escopo

A imagem atual configura apenas comunidade RO no `snmpd`; não configura `snmp-server host`, habilitação de traps ou destino de INFORM, pois essas diretivas dependem do equipamento/implementação e a imagem é um agente/emissor genérico. Os comandos `snmp-server ...` do pedido são específicos de CLI Cisco e não devem ser aplicados cegamente ao container ou a modelos desconhecidos.

**Risco de escopo do MVP:** foco e receptor MVP são SNMP v2c. Não há equipamento real identificado cuja operação v2c tenha sido comprovada. Se algum dispositivo do lab não operar em v2c (por exemplo, só falar v1 ou v3), registrar modelo, evidência e impacto: ele não poderá enviar eventos ao receptor sem adaptação/upgrade. Não é objetivo validar v1/v3 nos demais equipamentos. A aceitação de inventário com evidência real e de pelo menos um `.pcap` permanece pendente (depende de SCRUM-23/lab disponível).
