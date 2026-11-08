FROM alpine:3.20

RUN apk add --no-cache net-snmp net-snmp-tools iputils-ping iproute2 netcat-openbsd
RUN printf "rocommunity public\nsysLocation Lab_Cylock_UFS\n" > /etc/snmp/snmpd.conf

CMD ["snmpd", "-f", "-Lo", "-c", "/etc/snmp/snmpd.conf"]