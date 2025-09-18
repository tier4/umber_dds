# Participantの生存確認
rtps 2.3 spec
> 8.5.3.3 The built-in Endpoints used by the Simple Participant Discovery Protocol
> Periodically, the SPDP examines the SPDPbuiltinParticipantReader HistoryCache looking for stale entries defined as those that have not been refreshed for a period longer than their specified leaseDuration. Stale entries are removed.

Participantの生存は最後にSPDPbuiltinParticipantReaderのHistoryCacheが更新されてから、つまりSPDP messageを最後に受信してから、leaseDurationだけ何も受信しなかった場合に、そのParticipantはSPDPbuiltinParticipantReaderのHistoryCacheから削除される(=ネットワークを離脱したと判断される)。

ParticipantのLivelinessを宣言する手段はSPDP messageの送信のみ。

leaseDurationはSPDP messageのPID_PARTICIPANT_LEASE_DURATIONで伝えられる。

DDS 1.4 spec
> 2.2.3.11 LIVELINESS
> The Service takes responsibility for renewing the leases at the required rates and thus, as long as the local process where a DomainParticipant is running and the link connecting it to remote participants remains connected, the entities within the DomainParticipant will be considered alive.

Participantがネットワークを離脱した場合、そのParticipantが持つEntity (Writer, Reader)の間のコネクションも失なわれる。一度コネクションが失われると、再度discovery protocolによりmatchしないと通信が再開されない。
Cyclone DDSのコネクションが失なわれたときの挙動を観察した結果、対応するEntityのProxyが削除され、Listenerの{Publication/Subscription}Matchedを通じてそのことがユーザに通知されていた。

# Entityの生存管理

## DataWriterListener::LivelinessLost
LivelinessLostに関する記述はDDS, RTPSのspecにほとんど無い。
Cyclone DDSの挙動を観察した結果、Writerは自身のLivelinessQos.lease_durationの間生存が宣言されなかった場合、DataWriterListenerのLivelinessLostを通じてユーザに通知されると結論付けた。
一定期間生存が宣言されなかった場合、Listenerを通じてユーザに通知されるのみで他のアクションは確認できなかった。

DDS 1.4 spec
> 2.2.3 Supported QoS
> table: LIVELINESS
> The “liveliness” status of an Entity is used to maintain instance ownership in combination with the setting of the OWNERSHIP QoS policy.

おそらく、OWNERSHIP QoSのためにこれが必要であると思われる。

## Writer Liveliness Protocol
Writerが自身の生存 (Liveliness) を宣言するためのプロトコル。
これを元にReaderがremote Writerの生存の確認する。

ReaderはSEDP messageのPID_LIVELINESSに含まれるlease_durationの間Livelinessが更新されなかった場合、そのWriterはLivelinessを失なったと判断され、DataReaderListenerのLivelinessChangedを通じてユーザに通知される。
WriterのLivelinessを失なわれても、WriterProxyの削除は行なわれず、再度Writerの生存の宣言を確認した場合コミュニケーションは再開される。

## AUTOMATIC_LIVELINESS_QoS

rtps 2.3 sepc
> 8.4.13.5 Implementing Writer Liveliness Protocol Using the BuiltinParticipantMessageWriter and BuiltinParticipantMessageReader

Participantが1つ以上のAUTOMATIC_LIVELINESS_QOSを持つWriterを持つ場合、そのQoSを持つWriterの中で最小のlease durationよりも早い間隔で1つのサンプルを(ParticipantMessageWriterに)書き込むことでWriterのLivelinessを自動で宣言する必要がある。

DDS 1.4 spec
> 2.2.3.11 LIVELINESS
> The AUTOMATIC liveliness setting is most appropriate for applications that only need to detect failures at the process-level, but not application-logic failures within a process. The Service takes responsibility for renewing the leases at the required rates and thus, as long as the local process where a DomainParticipant is running and the link connecting it to remote participants remains connected, the entities within the DomainParticipant will be considered alive.

ただし、そのWriterのDataの書き込み頻度が十分に高い場合はParticipantMessageWriterへの書き込みは必要無い。

## MANUAL_BY_xxx_LIVELINESS_QoS

DataWriter::assert_liveliness()により、WriterのLivelinessを手動で宣言。

DDS 1.4 spec
> 2.2.2.4.2.22 assert_liveliness
> NOTE: Writing data via the write operation on a DataWriter asserts liveliness on the DataWriter itself and its DomainParticipant. Consequently the use of assert_liveliness is only needed if the application is not writing data regularly.

ただし、これもWriterのData書き込み頻度が低い場合のみ。

## WriterのLivelinessが更新される条件

### そのWriterからDataを受信した場合

+ そのWriterのLiveliness QoSがMANUAL_BY_{PARTICIPANT/TOPIC}_LIVELINESS_QOSに設定されている場合

DDS 1.4 spec
> 2.2.3.11 LIVELINESS
> The MANUAL settings (MANUAL_BY_PARTICIPANT, MANUAL_BY_TOPIC), require the application on the publishing side to periodically assert the liveliness before the lease expires to indicate the corresponding Entity is still alive. The action can be explicit by calling the assert_liveliness operations, or implicit by writing some data.

+ そのWriterのLiveliness QoSがAUTOMATIC_LIVELINESS_QOSに設定されている場合

rtsp specにはBuiltinParticipantMessageWriterを通じてlivelinessが管理されるとある。

rtps 2.3 spec
> 8.7.2.2.3 LIVELINESS
> DDS_AUTOMATIC_LIVELINESS_QOS : liveliness is maintained through the BuiltinParticipantMessageWriter.
> For a given Participant, in order to maintain the liveliness of its Writer Entities with LIVELINESS QoS set to
> AUTOMATIC, implementations must refresh the Participant’s liveliness (i.e., send the ParticipantMessageData, see 8.4.13.5) at a rate faster than the smallest lease duration among the Writers.

しかしDDS specにはAUTOMATIC livelinessは「プロセスレベルでの障害検出のみを必要とするアプリケーションに最適で、プロセス内のアプリケーションロジックに関する障害検出には適していません」とある。
この目的の達成のためにはあるWriterからDataを受信した場合、そのWriterは生存していると判断するので十分である。
またデータの送信頻度がlease durationよりも十分に短い場合、明示的にParticipantMessageDataによるlivelinessの宣言を行なわないという実装でCyclone DDS, Fast DDSとの通信テストした結果、livelinessのlostが発生しなかった。
そのため、本実装はAUTOMATIC livelinessの場合についてもあるWrtierからDataを受信した場合にそのWriterのlivelinessを更新するようにしている。

DDS 1.4 spec
> 2.2.3.11 LIVELINESS
> The AUTOMATIC liveliness setting is most appropriate for applications that only need to detect failures at the process-level, but not application-logic failures within a process.

### そのWriterが所属するParticipantのParticipantMessageDataを受信した場合
ただし、そのWriterのLiveliness QoSとParticipantMessageDataのkindが一致している場合のみ

rtps 2.3 sepc
> 8.4.13.5 Implementing Writer Liveliness Protocol Using the BuiltinParticipantMessageWriter and BuiltinParticipantMessageReader
> BuiltinParticipantMessageWriter and BuiltinParticipantMessageReader The liveliness of a subset of Writers belonging to a Participant is asserted by writing a sample to the BuiltinParticipantMessageWriter. If the Participant contains one or more Writers with a liveliness of AUTOMATIC_LIVELINESS_QOS, then one sample is written at a rate faster than the smallest lease duration among the Writers sharing this QoS. Similarly, a separate sample is written if the Participant contains one or more Writers with a liveliness of MANUAL_BY_PARTICIPANT_LIVELINESS_QOS at a rate faster than the smallest lease duration among these Writers. The two instances are orthogonal in purpose so that if a Participant contains Writers of each of the two liveliness kinds described, two separate instances must be periodically written. The instances are distinguished using their DDS key, which is comprised of the participantGuidPrefix and kind fields. Each of the two types of liveliness QoS handled through this protocol will result in a unique kind field and therefore form two distinct instances in the HistoryCache.
> In both liveliness cases the participantGuidPrefix field contains the GuidPrefix_t of the Participant that is writing the data (and therefore asserting the liveliness of its Writers).

### Liveliness Flagが立ったHeartBeatを受信した場合
ただし、そのWriterのLiveliness QoSがMANUAL_BY_{PARTICIPANT/TOPIC}_LIVELINESS_QOSに設定されている場合のみ

rtps 2.3 sepc
> 8.3.7.5 Heartbeat, Table 8.38 - Structure of the Heartbeat Submessage
> LivelinessFlag: Indicates that the DDS DataWriter associated with the RTPS Writer of the message has manually asserted its LIVELINESS.

Liveliness Flagが立ったHeartBeathは、DataWriter::assert_livelinessにより送信される。

DDS 1.4 spec
> 2.2.2.4.2.22 assert_livelines
> This operation manually asserts the liveliness of the DataWriter. This is used in combination with the LIVELINESS QoS policy (see 2.2.3, Supported QoS) to indicate to the Service that the entity remains active
> This operation need only be used if the LIVELINESS setting is either MANUAL_BY_PARTICIPANT or MANUAL_BY_TOPIC. Otherwise, it has no effect.

DataWriter::assert_livelinessが必要なのは、LIVELINESSの設定がMANUAL_BY_PARTICIPANTまたはMANUAL_BY_TOPICの場合のみで、それ以外の場合影響は無い。

### Liveliness QoSがMANUAL_BY_PARTICIPANT_LIVELINESS_QOSに設定されているWriterのLiveliness が更新された場合、そのWriterのParticipantが持つ他のWriterのlivelinessも更新される

DDS 1.4 spec
> 2.2.3.11 LIVELINESS
> The two possible manual settings control the granularity at which the application must assert liveliness.
> ・ The setting MANUAL_BY_PARTICIPANT requires only that one Entity within the publisher is asserted to be alive to deduce all other Entity objects within the same DomainParticipant are also alive.

