# SpecNote
RTPS 2.3 and DDS 1.4のSpecificationを読んだメモ+参考実装を読んだメモ

## 各DDS, RTPSエンティティーの役割
### Publisher/Subscriber (DDS)
DDS spec 2.2.2.4.1 Publisher Class
> A Publisher is the object responsible for the actual dissemination of publications.

RustDDSのsrc/dds/pubsub.rsのDDS Publisherのdocコメント
> The Publisher and Subscriber structures are collections of DataWriters
> and, respectively, DataReaders. They can contain DataWriters or DataReaders
> of different types, and attacehd to different Topics.

DataWriter/DataReaderを生成するためのもの
### DataWriter/DataReader (DDS)
#### DataWriter
DDS spec 2.2.2.4.2 DataWriter Class
> DataWriter allows the application to set the value of the data to be published under a given Topic

data: Dを受け取って、シリアライズしてSerializedPayloadを作成。作成したSerializedPayloadとオプションをチャネル:witer_commandを通じてRTPSWriterに渡す。
送信したい内容をRTPSWriterに渡す

### RTPSWriter/RTPSReader
### RTPSWriter
spec 8.4.2.2 Required RTPS Writer Behaviorを参照
RustDDSのsrc/dds/writer.rsのprocess_writer_commandのコメントの要約
1. DataWriterから受け取ったものをHistoryCacheに追加。
2. データを送信\
    データをpublishしたときはDATA submessageとHEARTBEAT submessageを送信\
    データをpublishしなかったときはHEARTBEAT submessageをだけを送信。このとき、Readerが興味を持っていればDATAとACKNACKを要求してくるはず

## Stateful/Stateless
rtps 2.3 spec 8.4.1 Overview, 8.4.3 Implementing the RTPS Protocol

RTPSの実装はメモリ使用量、帯域使用量、スケーラビリティ、効率の間で異なるトレードオフを選択することとなる。
RTPSの仕様は一致する振る舞いに対して１つの実装を命じない。
代わりにinteroperabilityのために最小の必要条件を定義し、Stateful, Statelessの2つのリファレンス実装を提供する。

### Stateless Reference Implementation:
スケーラビリティに最適化されている。ほとんどリモートエンティティーの状態を保持シないため、巨大システムにおいて良くスケールする。
これは、スケーラビリティを向上させ、メモリ使用量を削減するが、帯域使用量を増加させる。
Statelessな実装はmulticastを通じたbest-effortなコミュニケーションに向いている。
参照: rtps spec 8.4.7.2

### Stateful Reference Implementation:
リモートのエンティティーのすべての状態を保持する。
帯域使用量を最小化するアプローチだが、メモリ使用量か増加し、スケーラビリティが低下する。
Statelessな実装に比べて、厳密で信頼できるコミュニケーションを保証し、Writer側でのQoS-basedもしくはcontent-basedなフィルタリングが可能になる。
参照: rtps spec 8.4.10.2

## Topic
RustDDSでは、src/dds/topic.rsで定義されている。
~~spec探しても情報が見つからん。~~ RTPSじゃなくてDDSのsepcに情報があった。コードのコメントにも"DDS spec 2.3.3"って書いてあるのにRTPSのspecみてた。(https://www.omg.org/spec/DDS/1.4/PDF#G5.1034386)
僕 「Topicって何？」
Chat GPT4 「Topicは、名前（文字列）とデータ型を持ちます。」
RustDDSの実装を確認するとTopicにもたせてるのは名前, DomainParticipant, Qos, Kind(WITH_KEY or NO_KEY)で、データ型は含まれてない。
データ型とTopicを結びつけてるのは、publisher.create_datawriter_cdr::<Shape>(&topic, None)だと思われる。
// TODO: FastDDSがどうやってTopicとデータを結びつけてるのか調べる。
DDSHelloWorldのパケットキャプチャを解析した結果、RTPSを通じてやり取りされるのはTpicの名前のみらしい。具体的なデータ型はソースコードレベルで共有しておいて、
それに紐付いた名前のみをTopicに持たせるっぽい。

(https://fast-dds.docs.eprosima.com/en/latest/fastdds/dds_layer/topic/instances.html)
Topicは1つのデータタイプと紐付けられる。そのため、Topicと関係するデータサンプルはデータ型で示される情報のupdateとして理解される。しかし、論理的に分離して、同じトピック内に、同じデータ型を参照する複数のインスタンスを持つことも可能である。したがって、受信したデータサンプルは、そのTopicの特定のインスタンスに対する更新となる。

## QOS
https://www.omg.org/spec/DDS/1.4/PDF#G5.1034386

## GUID
RTPSEntity トレイトを実装してるやつは持ってる
Participant: 持ってる, RTPSEntity実装
Publisher: EntityIdだけ持ってる
Subscriber: 持ってない
RTPSWriter: 持ってる, RTPSEntity実装
RTPSReader: 持ってる, RTPSEntity実装
DDSWriter: 持ってる, RTPSEntity実装
DDSReader: 持ってる, RTPSEntity実装
### Prefixの決め方
とにかくDomainParticipantのguidから参照

    先頭2 octetはvenderIdの先頭2 octetと同じにする。これによってDDS Domain内で複数のRTPS実装が使われてもguidが衝突しない。残りの 10 octetは衝突しなければどんな方法で生成してもいい。(p. 144)

    RTPS spec 8.2.4.3 The GUIDs of the RTPS Endpoints within a Participant
    > The GUIDs of all the Endpoints within a Participant have the same prefix.

    あるParticipantに含まれるすべてのEndpointのGUIDは同じprefixを持つ。
    > The GUID of any endpoint can be deduced from the GUID of the Participant to which it belongs and its entityId.

    すべてのEndpintのGUIDは所属しているParticipantのGUIDとそのEndpointのentityIdから決定される。

### EntityId, EntityKindの決め方
TODO: EntityKindのそれぞれどう使えばいいのか調べる
GUIDはglobalにユニークなIDでGUID_Prefix, EntityKey, EntitiyKindで構成される。
GUID_Prefixはdomain_participantを表すものである。
したがって、同一participant内でEntityKey, EntitiyKindの組が一意であれば良い。
EntityKindのEntityのよって決定されるため、EntityKeyが同一participant内で一意になっていれば、
GUIDはglobalにユニークになる。
つまり、EntityKeyはparticipant内で一意になればどのように決めてもいい。

RTPS spec 8.2.4.4 The GUIDs of Endpoint Groups within a Participant
https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A103%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C406%2C0%5D
にPublisher, SubscriberはGUIDを持つと書いてある。

~~TODO: Publisher, SubscriberにGUIDを実装~~

RustDDSの実装で、DataWriterとRTPSWriterのGUIDが同じになるようになってるように見える
RTPSのspec 8.2を読んだ感じDDSWriter, DDSReaderはGUIDを持つ必要なさそう。
RustDDSの実装でDDSWriterがRTPSWriterと同じguidを持ってる理由は
1. DDSWtiterがdropするときに一緒に対応するDDSWrtierをドロップするため。
2. write_with_optionsの戻り値(writeから呼び出されるが戻り値は参照されてないので必要ない)
DDSWriterに対してはdropするときに必要になればGUIDを実装すればいいと思う。

RTPS 2.3 spec 8.2.1 Overview
> Each RTPS Entity is in a one-to-one correspondence with a DDS Entity.

RTPSWriterと(DDS)DataWriterは、１体１対応している。

### Message Receiverが従うルール (spec 8.3.4.1)
1. full Submessage headerを読み込めない場合、残りのMessageは壊れていると考える
2. submessageLengthフィールドは次のsubmessageがどこから始まるかを定義する、もしくは、Section 8.3.3.2.3(p. 34)で示されるようにMessageの終わりを拡張するSubmessageを指し示す。もしこのフィールドが無効なら、残りのMessageは無効である。
3. 未知のSubmessageIDをもつSubmessageは無視されなければならず、次のSubmessageに継続してパースされなければならない。具体的にRTPS 2.4の実装ではversion 2.4で定義されているSubmessageKind以外のIDをもつSubmessageは無視される。
未知のvenderId由来のvender-specificの範囲のSubmessageIdも無視されなければならず、次のSubmessageに継続してパースされなければならない。
4. Submessage flags.Submessageのreceiverは未知のflagを無視されるべきである。RTPS2.4の実装では"X"(unused)とプロトコルにマークされたすべてのフラッグは飛ばされるべきである。
5. 正しいsubmessageLengthフィールドは既知のIDをもつSubmessageであっても、常に次のSubmessageを探すのに使われなくてはならない。(おそらく、既知の種類のSubmessageで長さがわかっている場合でも、versionが上がって新しくElementが追加されている可能性があるから)
6. 既知だが、無効なSubmessageは残りのMessage(the rest of the Message)を無効にする。
// "the rest of the Message"が何を指すのか仕様書から読み取れないが、RustDDSの実装は、無効なSubmessageが含まれるMessageを無効なものとして破棄している
tomiy(tomiy-tomiylab)とytakano(ytakano)の解釈はそれまでに処理したSubmessageは使用し、無効なSubmessageとそれより後のSubmessageを破棄する。
ただし、無効なSubmessageを受け取ると、それ以降のSubmessageを無効とするとしか仕様書には書いておらず、Submessageを無効とする具体的な操作は定義されていない。
同一Message内に複数のSubmessageが含まれている場合、前のSubmessageは後ろのSubmessageを処理するのに必要な情報である。
"8.3.4 The RTPS Message Receiver
The interpretation and meaning of a Submessage within a Message may depend on the previous Submessages contained
within that same Message. "
つまり、Message内に1つでも無効なSubmessageが含まれている場合、そのMessageを処理する意義は失われるため、RustDDSでは破棄していると思われる。

## Example Behavior (日本語訳)
specのFigure 8.14 – Example Behavior

https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A193%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C489%2C0%5D

1. DDSユーザーがDDS DataWriterのwriteオペレーションを呼び出してデータを書き込む。
2. DDS DataWriterが新しいCacheChangeを作るために、RTPS Writerのnew_changeオペレーションを呼び出しす。
3. new_change オペレーションがrutenする。
4. DDS DataWriterがRTPS WriterのHistoryCacheにCacheChangeを保存するためにadd_changeを使う。
5. add_changeオペレーションがrutenする。
6. writeオペレーションがreturnする。
7. RTPS WriterがCacheChangeの変更内容をRTPS ReaderにData Submessageを使って送信し、Heartbeat Submessageを送信してacknowledgemntを要求する。
8. RTPS ReaderがData messageを受信し、リソースの制限が許すと仮定し、add_changeオペレーションを使ってreaderのHistoryCacheにCacheChangeを配置する。
9. add_changeオペレーションがruturnする。CacheChangeはDDS DataReaderとDDSユーザーから見える。この条件はRTPS ReaderのreliabilityLevelアトリビュートに依存する。

    a.  RELIABLE DDS DataReaderには、 RTPS ReaderのHistoryCacheにあるchangeはすべてのそれより前のchange(i.e., より小さいsequence numberをもつchange)が見える場合のみ、ユーザーアプリケーションから見えるようになる。

    b. BEST_EFFORT DDS DataReaderには、RTPS ReaderのHistoryCacheにあるchangeは未来のchangeがまだ見えるようになっていない(i.e., RTPS ReceiverのHistoryCacheにより大きいsequence numberをもつchangeがない)場合のみ、ユーザーから見えるようになる。

〜〜続く〜〜

// TODO

## 8.5 Discovery Module (日本語訳, 要約)
Discovery Moduleはconfigがどのように行われるのか仮定を行わず、Endpoint間でどのようにデータが交換されるかのみ定義される。Endpointの設定をするために、実装は存在するremote Endpointの情報とそのpropertieをてに入れないといけない。この情報をどのように獲得するかがDiscovery Moduleのテーマである。

Discovery ModuleはRTPS discovery moduleを定義する。discovery protocolの目的はRTPS Participantが関係する他のParticipantとEndpointを発見すること。一度、remote Endpointが発見されれば、実装はlocal Endpointをコミュニケーションを樹立するための設定ができる。

DDSの仕様は一致するDataWriterとDataReader間のコミュニケーションの樹立はdiscovery mechanismに頼っている。DDS実装はremote entityの存在をnetworkに参加したときと離れたときの両方を自動で発見しなければならない。discovery情報はDDS built-in topicを使うことでアクセス可能になる。

Discovery Moduleで定義されるRTPS discovery protocolはDDSが必要とするdiscovery mechanismを提供する。

### Overview
discovery protocolは独立した２つのプロトコル、Participant Discovery Protocol(PDP), Endpoint Discovery Protocol(EDP)によって構成される。PDPはnetwork上でどのようにParticipantがお互いを発見するかを決めている。一度2つのParticipantがお互いを発見すると、それらはParticipantが保持しているEndpointが持つ情報をEDPを使って交換する。この因果関係を除けば、両プロトコルは独立したものとみなすことができる。

実装によっては、ベンダー固有の可能性も考慮して、複数のPDPおよびEDPをサポートすることを選択できます。一般的に、2つのParticipantは少なくとも1つのPDPとEDPを持つから、それらは必要なdiscovery informationを交換することができる。interoperabilityのために、すべてのRTPS実装は少なくともSimple Participant Discovery Protocol(SPDP)とSimple Endpoint Discovery Protocl(SEDP)を提供しなければならない。

両方の基本的なdiscovery protocolは小規模から中規模のnetworkにおいて十分足りる。大規模なnetworkのためのadditional PDPs and EDPsは将来のversionの仕様で追加されるかもしれない。

discovery protocolの役割は発見されたremote Endpointに情報を提供することである。Participantがその情報を自身が持つEndpointを設定するためにどのように使われるかは、実際のRTPS protocolの実装に依存し、discovery protocolの仕様の一部ではない。例えば、8.4.7で紹介されているrefarence実装では、remote Endpointで獲得した情報は以下を設定することを可能にする。
+ The RTPS ReaderLocator objects that are associated with each RTPS StatelessWriter.
+ The RTPS ReaderProxy objects associated with each RTPS StatefulWriter
+ The RTPS WriterProxy objects associated with each RTPS StatefulReader

Discovery Moduleの構成
+ SPDPとSEDPはdiscovery informationを交換するために事前に提示されたRTPS built-in WriterとReaderを使用するので、8.5.2でそれらのRTPS built-in Endpointを紹介する。
+ The SPDP is discussed in 8.5.3.
+ The SEDP is discussed in 8.5.4.

### 8.5.2 RTPS Built-in Discovery Endpoints
“DCPSParticipant”, “DCPSSubscription”, “DCPSPublication”, “DCPSTopic”の4つの事前に定義されたbuilt-in Topicがある。これらのTopicに関係するDataTypeはDDSで決められており、主にEntity QoS valueが格納される。

それぞれのbuilt-in Topicについて、それぞれに一致するDDS built-in DataWriterとDataReaderが存在する。buiilt-in DataWriterは存在と、local DDS ParticipantのQoSとそれが持っているDataReader, DataWriter, Topicなどを残りのnetworkに伝えるために使われる。さらに、built-in DataReaderはこの情報を一致するremote EntityのDDS実装を特定するためにremote Participantから集める。built-in DataReaderは通常のDDS DataReaderと同じように振る舞い、DDS APIを通じてuserがアクセスすることができる。

RTPS Simple Discovery Protocol(SPDP and SEDP)がとっているアプローチはbilt-in Entityコンセプトと似ている。RTPSはそれぞれのbuilt-in DDS DataWriterとDataReaderに関連するbuilt-in RTPS Endpointを割り当てている。それらのbuilt-in Endpointは通常のWriter, Reader Enddpointのように振る舞い、Behavior Moduleで定義される通常のRTPS protocolを使用してParticipant間で必要なdiscovery informationを交換する方法を提供する。

### 8.5.3 The Simple Participant Discovery Protocol
PDPの目的はnetwork上の他のParticipantを発見し、そのpropertyを取得すること。Participantは複数のPDPをサポートしているかもしれないが、interoperabilityのためにすべての実装は少なくともSPDPをサポートしなければならない。

### 8.5.3.1 General Approach
SPDPはdomainに含まれるParticipantの存在を知らせたり、検知するのにシンプルなアプローチを使用する。

それぞれのParticipantでSPDPは2つのRTPS built-in Endpoints、SPDPbuiltinParticipantWriterとSPDPbuiltinParticipantReaderを作成する。

SPDPbuiltinParticipantWriteはRTPS Best-Effort StatelessWriter。SPDPbuiltinParticipantWriterのHistoryCacheはa single data-object of type SPDPdiscoveredParticipantDataを含む。このdata-objectの値はParticipantのatributeからセットされる。もし、atributeが変更されればdata-objectは交換される。

SPDPbuiltinParticipantWriterは定期的にdata-objectを事前に設定されたlocatorのリストにParticipantの存在を知らせるためにnetworkに送信する。これはStatelessWriterのHistoryCacheに存在するすべてのchangesをすべてのlocatorに送信するStatelessWriter::unsent_changes_resetを定期的に呼び出すことで達成される。SPDPbuiltinParticipantWriterがSPDPdiscoveredParticipantDataを送信する周期のdefaultはPSMで決定されている。その周期はSPDPdiscoveredParticipantDataで決められるleaseDurationよりも小さくするべきである。(see also
8.5.3.3.2)

事前に設定されたlocatorのリストはunicastとmulticastの両方のlocatorを含んでいる可能性がある。port番号はそれぞれの
PSMで定義される。これらのlocatorは単にnetwork上にいるかもしれないremote Participantを表しており、Participantが実際に存在する必要はない。SPDPdiscoveredParticipantDataを定期的に送信することにより、Participantはnetworkにどの順番でも参加できる。

SPDPbuiltinParticipantReaderはremote ParticipantからSPDPdiscoveredParticipantData announcementを受信する。そのテータにはremote ParticipantがどのEndpoint Discovery Protocolをサポートしているかの情報が含まれている。適切なEndpoint Discovery Protocolはremote Particpnat同士がEndpointの情報を交換するために使用される。

実装は未知であったParticipantから受信したSPDPdiscoveredParticipantData data-objectに対する返事で追加のSPDPdiscoveredParticipantDataを送信することでany start-up delaysを最小化することができる。しかし、この振る舞いは任意である。実装はユーザーにpre-configured locatorのリストを新たに発見されたParticipantを追加して拡大するかどうかを選択できるようにできるかもしれない。これはa-symmetricなlocatort listを可能にする。これらの最後の2つの機能は任意でinteroperabilityのためには必要ではない。