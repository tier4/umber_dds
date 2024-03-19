# SpecNote
RTPS 2.3 and DDS 1.4のSpecificationを読んだメモ+参考実装を読んだメモ

## WARNING 注意
RTPS 2.3 specに登場する'long'は4 byte = 4 octet = 32bitであることに注意！

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
```
struct GUID {
    guid_prefix: GuidPrefix,
    entity_id: EntityId,h
}

struct EntityId {
    entity_key: [u8; 3],
    entity_kind: EntityKind,
}
```
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

### domainId/participantId
rtps 2.3 spec 9.6.1.1 Discovery traffic
> The domainId and participantId identifiers are used to avoid port conflicts among Participants on the same node.
> Each Participant on the same node and in the same domain must use a unique participantId. In the case of multicast, all
> Participants in the same domain share the same port number, so the participantId identifier is not used in the port number
> expression.
domainId, participantIdは同一ノード上のParticipantのport番号がかぶるのを防ぐためのもの。
participantIdは同一ノード上の同一ドメインのParticipantの中で一意でないといけない。

### Participant数の制限
rtps 2.3 spec 9.6.1.1 Discovery traffic
> PB = 7400
> DG = 250
> 省略
> Given UDP port numbers are limited to 64K, the above defaults enables the use of about 230 domains with up to 120
> Participants per node per domain.
UDPポート番号に使えるのは最大で64K(16bit: 0-65535)である。
デフォルトのUnicast User trafficのポート番号が、domainIdが227, participantIdが120のとき65470になる。
OpenSpliceのparticipant数制限の由来はおそらくこれ。

## 8 Platform Independent Model (PIM)
PIMはプロトコルを"virtual machine"という用語を用いて説明する。
RTPS virtual machineを紹介する唯一の目的はプロトコルを完璧で一意なやり方で説明することである。この説明は内部実装を強制する意図はない。唯一の完全な実装の基準は外部から観測される振る舞いがインターオペラビリティのための必要条件を満足することである。特に、実装は他のクラスをベースにすることもでき、RTPSプロトコルを実装するためにステートマシン以外のプログラミング構造を使用することもできる。

### SequenceNumberSet
```
typedef sequence<long, 8> LongSeq8;
struct SequenceNumberSet {
    SequenceNumber_t bitmapBase; // bitmapのLSBと対応するSequenceNumber
    LongSeq8 bitmap; // 4 octet * 8で256bitのbitmap
};
```
The structure offers a compact representation encoding a set of up to 256 sequence numbers.
base から base + 256の範囲で含まれるSequenceNumberを表すのに、
含まれるSequenceNumberをリストアップすると、N*sizeof(SequenceNumber)だけ必要になり、可変長のメモリが必要になる。
この方式だと、256bit + sizeof(SequenceNumber)の固定長だけで済む。

### Endiannessのについて
rtps 2.3 spec 9.4.5.1.1.1 Submessage Ranges Reserved by other Specifications
> Sub clause 8.3.3.2 in the PIM defines the EndiannessFlag as a flag present in all Submessages that indicates the
> endianness used to encode the Submessage.
EndiannessはSubMessageのエンコードに使われたエンディアンでCPUのエンディアンではない。

## 8.2.2 The RTPS HistoryCache
DDSとRTPSの間のインターフェースで、readerとwriterで異なる役割を果たす。
writer 側では、一致するDDS Writerで作られたdata-objectに対するcahngesを保存する。これは、既存の、もしくは将来matchするRTPS Readerにサービスを提供するために必要である。どのhistoryが必要かはDDS QoSと、macheしたRTPS Readerとのコミュニケーションの状態に依存する。

reader側では、すべての一致するRTPS writerで作られたdata-objectsに対するchangesが積み重なった一部を保存する。

これまでにつくられたすべてのchangesのすべての履歴を保持する必要はない。

むしろ必要なのは、RTPSプロトコルの動作ニーズと関連するDDSエンティティのQoSニーズを満たすために必要な履歴のサブセットである。このサブセットを定義するルールはRTPSプロトコルによって定義され、通信プロトコルの状態と関連するDDSエンティティのQoSの両方に依存する。

HistoryCacheはDDSとRTPSのインターフェースだから、RTPSとそれに関連するDDSのエンティティーはどちらも、紐付けられたHistoryCacheの操作できる。

sample bihaviorの図をみると誰がHistoryCacheを持っているのかわからないけど、rtps spec 2.4の8.4.7.1 を見ると、RTPS Writerがwriter cache を持っていることがわかる。

## 8.4 Behavior Module
このモジュールではRTPS entityの動的な振る舞いを説明する。RTPS WriterとRTPS Readerの間でメッセージを交換する正しい流れとそれらを構築するタイミングを説明する。

### 8.4.1 Overview
一度RTPS WriterがRTPS Readerとマッチすると、それらはWriterのHistoryCacheに存在するCacheChangeの変更をReaderのHistoryCacheに伝播させることを保証しなければならない。

Behavior ModuleはどのようにマッチするRTPS WriterとReaderのペアがCacheChangeの変更を伝播させるためにどのように振る舞うべきか説明する。8.3で定義されるRTPS Messageをつかったメッセージの交換として振る舞いは定義される。

Behavior Moduleは以下のように構成されている。
- 8.4.2は振る舞いに関してすべてのRTPSプロトコル実装がが満足するべき必要条件を列挙している。それらの必要条件を満足する実装はcompliantでほかのcompliantな実装とinteroperableであるといえる。
- 上で暗示されているように、複数の実装は最小の必要条件を満足していれば、それぞれの実装はmemory requirements, bandwidth usage, scalability, efficiencyの間で異なるトレードオフを選択できる。RTPSの仕様では対応する振る舞いを持つ1つの実装を要求しない。代わりに、インターオペラビリティのための最小の必要条件とStatelessとStatefulという2つのリファレンス実装を8.4.3で提供する。
- プロトコルの振る舞いはRELIABILITY QoSのような設定に依存する。8.4.4では利用可能な組み合わせを説明する。
- 8.4.5, 8.4.6では表記の慣例とこのモジュールで仕様される新しい型を定義する。
- 8.4.7から8.4.12まで2つのリファレンス実装をモデル化する。
- 8.4.13ではParticipantsがそれぞれが保持しているWriterのlivelinessを宣言するために使われるWriter Liveliness Protocolを説明する。
- 8.4.14ではfragmented dataを含む、いくつかの補足的な振る舞いを説明する。
- 最後に、8.4.15は正い実装のためのガイドラインを提供する。

### 8.4.1.1 Example Behavior (日本語訳)
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
// TODO
23.

上記の説明はいくつかのDDS DataReaderとRTPS Readerの間のやり取りがモデル化されていない。例えば、RTPS Readerが使うDataReaderに新しいchangeが受信されたかどうか確認するために`read`もしくは`take`を呼ぶ(i.e., what causes step 10 to be taken)べきであることを知らせる仕組みはモデル化されていない。

また、いくつかのDDS DataWriterとRTPS Writerの間のやり取りもモデル化されていない。例えば、RTPS Writerが使うDataWriterに特定のchangeがacknowledgedが完了していて、HistoryCacheaから削除可能かどうか確認するように知らせる仕組みがモデル化されていない。

前述のやり取りがモデル化されていないのは、それらはmiddlewareの内部実装であり、RTPS protocolに影響を与えないからである。

### 8.4.2 Behavior Required for Interoperability
この章では全てのRTPS実装が
- プロトコルの仕様に準拠する
- 他の実装と相互使用可能にする

ために満足しなければならない必要条件を説明する。

それらの必要条件の範囲は異なるベンダーのRTPS実装の間でのメッセージ交換に限定される。
同じベンダー同士のメッセージ交換では、ベンダーは準拠していない実装を選択するかもしれないし、proprietaryなプロトコルを代わりに使うかもしれない

### 8.4.2.1 General Requirements
以下の必要条件はすべてのRTPS Entityに適用される。

#### 8.4.2.1.1 All communications must take place using RTPS Messages
8.3で定義されているRTPS Message以外は使用できない。必要なコンテンツ、正当性、それぞれのメッセージの解釈はRTPSの仕様で規定されている。

ベンダーはプロトコルによて提供されるextension mechanisms(8.6を参照)を使用してメッセージをベンダー特有のニーズのためにメッセージを拡張するかもしれない。これはinteroperoperabilityに影響しない。

#### 8.4.2.1.2 All implementations must implement the RTPS Message Receiver
RTPS Messageに含まれるSubmessageを解釈し、MessageReceiverの状態を保持するために、8.4.3で説明されるRTPS Message Receiverが従がうルールを実装しなければならない。

この必要条件には、8.3.7 に定義されているように、Entity Submessagesの適切な解釈のために必要な場合、Interpreter SubmessagesでEntity Submessagesを先行させることによる適切なメッセージフォーマットも含まれる。

#### 8.4.2.1.3 The timing characteristics of all implementations must be tunable
アプリケーションの要件、deployment configuration、および基礎となるトランスポートによって、エンドユーザーはRTPSプロトコルのタイミング特性を調整することを望むかもしれない。

したがって、プロトコルの動作に対する要求が、遅延応答を許容したり、周期的なイベントを指定したりする場合、実装はエンドユーザーがそれらのタイミング特性を調整できるようにしなければならない。

#### 8.4.2.1.4 Implementations must implement the Simple Participant and Endpoint Discovery Protocols
実装はremote Endpointsのdiscoveryを可能にするためにSimple Participant/Endpoint Discovery Protocolを実装しなければならない。

RTPSはアプリケーションのeployment needsによって異なるParticipant/Endpoint Discovery Protocolを使用することを許している。
interoperabilityのため、実装はすくなくともSimple Participant Discovery ProtocolとSimple Endpoint Discovery Protocolを実装しなければならない。(8.5.1を参照)

### 8.4.2.2 Required RTPS Writer Behavior
以下の必要条件はRTPS Writerのみに適用される。言及されない限り、必要条件はreliableとbest-effortの両方に適用される。

#### 8.4.2.2.1 Writers must not send data out-of-order
Writerはdata sampleをそれらがHistoryCacheに追加された順番で送信しなければならない。

#### 8.4.2.2.2 Writers must include in-line QoS values if requested by a Reader
Writerはdin-line QoSと共にdata messageを受信するために、Readerの要求に従がわなければならない。

#### 8.4.2.2.3 Writers must send periodic HEARTBEAT Messages (reliable only)
Writerは、利用可能なサンプルのシーケンス番号を含む定期的なHEARTBEAT Messageを送信することによって、データサンプルが利用可能であることを、マッチングする各Reliable Readerに定期的に通知しなければならない。もし、サンプルが存在しなければHEARTBEAT Messageを送信する必要はない。

厳格で信頼できるコミュニケーションのため、Writerは継続してHEARTBEAT MessageをReaderにReaderが利用可能なサンプルをすべて受信したことをacknowledgeするか、または受信したサンプルがなくなるまで送信しなければならない。それ以外のすべての場合において、送信される HEARTBEAT メッセージの数は実装に依存し、有限である可能性がある。

#### 8.4.2.2.4 Writers must eventually respond to a negative acknowledgment (reliable only)
Readerがdata sampleを失なったことを示すACKNACK Messageを受信したとき、Writerは失なわれたdata sampleを送信し、もしくはサンプルが関係ないときはGAP messageを送信し、もしくはサンプルがそれ以上利用可能でないならHEARTBEAT messageを送信しなければならない。

Writerは即座に反応するか、将来の特定の時間帯に応答をスケジュールすることを選択する。それはまた、関連する応答をまとめることも可能で、ACKNACK MessageとWriterのresponseが一対一で対応している必要はない。それらの決定とタイミング特性は実装特有である。

#### 8.4.2.2.5 Sending Heartbeats and Gaps with Writer Group Information
Groupに所属しているWriterはReaderがすべてのWriterのsampleにacknowledgeしているたとしても、HEARTBEAT or GAP SubmessagesをそのマッチしているReaderに送信するべき(shall)である。これは、Subscriverが、そのWriterで利用できないグループシーケンス番号を検出するために必要である。このルールの例外はWriterが同じ情報を含む DATA or
DATA_FRAG Submessagesを送信したときである。

### 8.4.2.3 Required RTPS Reader Behavior
best-effortなReaderは、データを受信するだけで、自分ではメッセージを送信しないので、完全に受動的である。したがって、以下のrequirementsはreliable Readerのみに適用される。

#### 8.4.2.3.1 Readers must respond eventually after receiving a HEARTBEAT with final flag not set
final flagがセットされていないHEARTBEAT Messageを受信したとき、ReaderはACKNACK Messageに応答しなければならない。ACKNACK Messageは、すべてのデータサンプルを受信したことを認めるか、またはいくつかのデータサンプルが欠落していることを示す。

応答はmessage storms(おそらく輻輳のこと)を避けるため遅延するかもしれない。

#### 8.4.2.3.2 Readers must respond eventually after receiving a HEARTBEAT that indicates a sample is missing
HEARTBEAT Messageを受信するとき、data samplesを失なったReaderはdata sampleが失なわれたことを示すACKNACK Messageに応答しなければならない。この要件は、Readerがそのキャッシュにこれらの欠落サンプルを収容できる場合にのみ適用され、 HEARTBEAT メッセージのfinal flagの設定とは無関係である。

応答はmessage storms(おそらく輻輳のこと)を避けるため遅延するかもしれない。

liveliness HEARTBEATが、livelinessのみのメッセージであることを示すために、liveliness flagsとfinal flagの両方がセットされている場合、この応答は必要ない。

#### 8.4.2.3.3 Once acknowledged, always acknowledged
一度Readerが受信したsampleにACKNACK Messageを使かって陽にacknowledgeした場合、それ以降その同じサンプルに対してnegative acknowledgeすることはできない。

一度WriterがすべてのReaderからpositive acknowledgeを受信すると、Writerは関連するリソースを再利用することができる。しかし、もしWriterが以前positively acknowledgeを受けたsampleに対し、negative acknowledgemenを受け、Writerがrequestをserviceしている場合、Writerはsampleを送信すべきである。

#### 8.4.2.3.4 Readers can only send an ACKNACK Message in response to a HEARTBEAT Message
定常状態では、ACKNACK MessageはWriterからのHEARTBEAT Messageに対する応答としてのみ送信できる。ACKNACK Messages can be sent from a Reader when it first discovers a Writer as an optimization.(???) Writerはそれらの先取りのACKNACK Messageに応答する必要はない。

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

## 8.4.15 Implementation Guidelines
この章は正式なプロトコルの仕様ではない。この章の目的は高パフォーマンスなプロトコル実装のためのガイドラインを提供することである。

### 8.4.15.1 Implementation of ReaderProxy and WriterProxy
PIMはWriter’s HistoryCacheのそれぞれのCacheChangeの関係を保持するReaderProxyをモデル化している。この関係はassociation class ChangeForReaderによって媒介されるものとしてモデル化される。直接このモデルを実装すると、それぞれのReaderProxyで大量の情報が保持されることになる。実際には、ReaderProxyが必要とするものはプロトコルに使われる操作を実装できるようにすることとで、正確な関係を使う必要はない。

たとえば、 unsent_changes()、 next_unsent_change()というオペレーションはhighestSeqNumSentという1つのSeqenceNumberをReaderProxyが保持していれば実装できる。highestSeqNumSentはReaderProxyに向けて送信したCacheChangeの中で最も大きいSequenceNumberを保持する。これを使えば、unsent_changes()というオペレーションはHistoryCacheのすべてのchangeを検索し、highestSeqNumSetよりも大きなsequenceNumberを持つものを選択すればいい。 next_unsent_change()の実装は、HistoryCachaを検索し、highestSeqNumSetより大きい、2番めに大きいserquenceNumber(the next-highest sequence number greater than highestSeqNumSent)をもつCacheChangeを返せば良い。これらのオペレーションはHistoryCacheがsequenceNumberをindexとして保持しているときに効率的になる。

同じテクニックがrequested_changes(), requested_changes_set(), and
next_requested_change(SequenceNumber_t lowestRequestedChange and a fixed-length bitmapで表現するのが効果的)の実装にも使える。このケースでは実装は、どの特定のsequenceNumberが現在requestされているかを保存するための1つのSequenceNumberの窓()を保持すればよい。. Requests that do not fit in the window can be ignored as they correspond to sequence numbers higher than the ones in the window and the reader can be relied on re-sending the request later if it is still missing the change.

似たテクニックがacked_changes_set() and unacked_changes()の実装にも使える。

## SPDPまとめ
builtinのRTPS reader, RTPS writerを作る
このreader, writerは“DCPSParticipant”Topicのデータをやり取りする。
readerはspdpメッセージを受け取って、discovery_db()にremote Participantの情報を入れる
すでに、discovery_db()に登録されているremote Participantからspdpメッセージを受け取ったらdiscovery_dbのそのparticpantのエントリーの最終更新時刻をアップデート
writerは一定間隔()でspdpメッセージを送信

discovery_dbは定期的にチェックして最終更新時刻からleaseDuration経過していたらそのエントリーを削除

writerはspdpメッセージを定期的に、新たにネットワークに自身の存在を伝えるためにマルチキャストで、既知のParticipantに対して自身の生存を伝えるためにユニキャストで送信する。

memo: SPDPのためのData submsgはWireshark上でDATA(p)と表示される。

### SEDPまとめ
最初にSPDPメッセージを受け取ったとき、そのメッセージの送信元に対してユニキャストでSEDPメッセージを送信する。

Endpointが変更されたときは、既知のParticipantに対して変更を知らせるためユニキャストでSEDPメッセージを送信する。

> 8.5.4.2 The built-in Endpoints used by the Simple Endpoint Discovery Protocol
> SEDP DDS built-in Entityは“DCPSSubscription,” “DCPSPublication,” と“DCPSTopic” Topicsを対応付ける。
> DDS specificationによると、それらのbulit-in Entityのreliablility QoSは'reliable'にセットされる。

memo: SEDPのためのData submsgはWireshark上でDATA({r|w})と表示される。(r)は送信元がreader, (w)は送信元がwriterであることを表す。

### reliableとbest-effortの違い
> 8.4.2.2 Required RTPS Writer Behavior
> Writers must send periodic HEARTBEAT Messages (reliable only)

reliableなWriterは定期的にHEARTBEAT Messageを送らなければならない。
> Writers must eventually respond to a negative acknowledgment (reliable only)

reliableなWriterはNACKに応答しなければならない。

## 8.5 Discovery Module (日本語訳, 要約)
Discovery Moduleはconfigがどのように行われるのか仮定を行わず、Endpoint間でどのようにデータが交換されるかのみ定義される。Endpointの設定をするために、実装は存在するremote Endpointの情報とそのpropertieをてに入れないといけない。この情報をどのように獲得するかがDiscovery Moduleのテーマである。

Discovery ModuleはRTPS discovery protocolを定義する。discovery protocolの目的はRTPS Participantが関係する他のParticipantとEndpointを発見すること。一度、remote Endpointが発見されれば、実装はlocal Endpointをコミュニケーションを樹立するための設定ができる。

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

SPDPbuiltinParticipantWriterは定期的にdata-objectを事前に設定されたlocatorのリストにParticipantの存在を知らせるためにnetworkに送信する。これはStatelessWriterのHistoryCacheに存在するすべてのchangesをすべてのlocatorに送信するStatelessWriter::unsent_changes_resetを定期的に呼び出すことで達成される。SPDPbuiltinParticipantWriterがSPDPdiscoveredParticipantDataを送信する周期のdefaultはPSMで決定されている。その周期はSPDPdiscoveredParticipantDataで決められるleaseDurationよりも小さくするべきである。(see also 8.5.3.3.2)

事前に設定されたlocatorのリストはunicastとmulticastの両方のlocatorを含んでいる可能性がある。port番号はそれぞれの
PSMで定義される。これらのlocatorは単にnetwork上にいるかもしれないremote Participantを表しており、Participantが実際に存在する必要はない。SPDPdiscoveredParticipantDataを定期的に送信することにより、Participantはnetworkにどの順番でも参加できる。

SPDPbuiltinParticipantReaderはremote ParticipantからSPDPdiscoveredParticipantData announcementを受信する。そのテータにはremote ParticipantがどのEndpoint Discovery Protocolをサポートしているかの情報が含まれている。適切なEndpoint Discovery Protocolはremote Particpnat同士がEndpointの情報を交換するために使用される。

実装は未知であったParticipantから受信したSPDPdiscoveredParticipantData data-objectに対する返事で追加のSPDPdiscoveredParticipantDataを送信することでany start-up delaysを最小化することができる。しかし、この振る舞いは任意である。実装はユーザーにpre-configured locatorのリストを新たに発見されたParticipantを追加して拡大するかどうかを選択できるようにできるかもしれない。これはa-symmetricなlocatort listを可能にする。これらの最後の2つの機能は任意でinteroperabilityのためには必要ではない。


### 8.5.3.3 The built-in Endpoints used by the Simple Participant Discovery Protocol
SPDPbuiltinParticipantReaderのHistoryCacheには、アクティブに検出されたすべてのParticipantの情報が含まれている。各データオブジェクトを識別するために使用されるキーは、ParticipantのGUIDに対応しています。SPDPbuiltinParticipantReaderがParticipantの情報を受信するたびに、ParticipantのGUIDと一致するキーを持つエントリを探すために、SPDPはHistoryCacheを検査する。一致するキーを持つエントリが存在しない場合、ParticipantのGUIDをキーとする新しいエントリが追加される。

SPDPは定期的に新鮮でないエントリー(leaseDurationで定められる期間よりも長い間更新されていないエントリー)を探すためにSPDPbuiltinParticipantReaderのHistoryCacheを検査する。新鮮でないエントリーは削除される。

### 8.5.3.4 Logical ports used by the Simple Participant Discovery Protocol
上で言及したように、それぞれのSPDPbuiltinParticipantWriterはParticipantの存在をネットワークに伝えるため、事前に設定されたlocatorのリストを使う。

plug-and-play interoperabilityの実現のため、事前に設定されたlocatorのリストは以下のwell-known logical portを使用しなければならない。

SPDP_WELL_KNOWN_UNICAST_PORT
    entries in SPDPbuiltinParticipantReader.unicastLocatorList,
    unicast entries in SPDPbuiltinParticipantWriter.readerLocators

SPDP_WELL_KNOWN_MULTICAST_PORT
    entries in SPDPbuiltinParticipantReader.multicastLocatorList,
    multicast entries in SPDPbuiltinParticipantWriter.readerLocators

実際のlogical portの値はPSMで定義される。
> 9.6.1.1 Discovery traffic を参照

## 8.5.4 The Simple Endpoint Discovery Protocol
Endpoint Discovery Protocol はお互いのWriterとReader Endpointを発見するために2つのParticipant間で交換する必要のある情報を定義している。

Participantは複数のEDPをサポートしているかもしれない。しかし、interoperabilityのためにすべての実装は少なくともSEDPをサポートしなければならない。

## 8.5.4.1 General Approach
SPDPと同じように、SEDPは事前に定義されたbuilt-in Endpointを使用する。

事前に定義されたbuilt-in Endpointを使用することは、一度Participantが他のParticipantの存在を知れば、remote participantによって利用可能なbuilt-in Endpointの存在を推定できるようになり、locally-matching built-in Endpointをつなげることを意味する。

built-in Endopint間で情報をやり取りするために使われるプロトコルはapplicationによって定義されたEndpointに使用されるものと変わらない。
したがって、 そのメッセージがbuilt-in Reader Endpointによって読まれることによって、 プロトコルvirtual machineはその存在や、どこかのremote Participantsに所属しているDDS EntityのQoSを発見することができる。built-in Writer Endpointが書き込むことによって、同様にParticipantは他のParticipantに存在とlocal DDS EntityのQoSを知らせることができる。

したがって、SEDPで組み込みtopicを使用すると、全体的なdiscovery protocolの範囲が、システム内にどのParticipantが存在するか、およびこれらのParticipantの組み込みEndpointsに対応するReaderProxyオブジェクトとWriterProxyオブジェクトの属性値を決定することに縮小される。
それがわかれば、あとはすべて、RTPSプロトコルを内蔵のRTPS readerとwriter間の通信に適用することで結果が得られる。

## 8.5.4.2 The built-in Endpoints used by the Simple Endpoint Discovery Protocol
SEDP DDS built-in Entityは“DCPSSubscription,” “DCPSPublication,” と“DCPSTopic” Topicsを対応付ける。
DDS specificationによると、それらのbulit-in Entityのreliablility QoSは'reliable'にセットされる。
したがって、SEDPはbuilt-in DDS DataWriter, DataReaderと一致するreliable RTPS Writer, Reader Endpointを結びつける。

たとえば、図 8.29せ説明されているように、 the DDS built-in DataWriters for the “DCPSSubscription,” “DCPSPublication,”
and “DCPSTopic” Topics can be mapped to reliable RTPS StatefulWriters and the corresponding DDS built-in
DataReaders to reliable RTPS StatefulReaders. 実際の実装ではstatefull refarence 実装を使う必要はない。
interoperabilityのため、実装はbuilt-in Endpointが必要とするものと、8.4.2に挙げられているgeneral requirementsを満たす
reliable communicationを提供すれば十分である。

## 8.5.4.3 Built-in Endpoints required by the Simple Endpoint Discovery Protocol
実装はすべてのbuilt-in Endpoint を提供する必要はない。

DDS specificationで触れられているように、Topic propagationは任意である。
したがって、SEDPbuiltinTopicsReader, SEDPbuiltinTopicsWriter built-in Endpointsを実装する必要はなく、
interoperabilityのため実装はremote Participantのそれらの存在に頼るべきではない。

残りのbuilt-in Endpointに関しては、Participantはlocal Endpointとremote Endpointのマッチングに必要なbuilt-in Endpointのみを提供する必要があります。
たとえば、DDS ParticipantがDataWriterしか保持していなければ、必要なRTPS built-in EndpointはSEDPbuiltinPublicationsWriterとSEDPbuiltinSubscriptionsReaderのみである。
このケースにおいて、SEDPbuiltinPublicationsReaderとSEDPbuiltinSubscriptionsWriter built-inEndpointsは何の目的も果たさない。

SPDPはどのようにParticipantが他のParticipantに利用可能なbuilt-in Endpoint知らせるかを規定する。
これは8.5.3.2で議論されている。

## 8.5.4.4 Data Types associated with built-in Endpoints used by the Simple Endpoint Discovery Protocol
それぞれのRTPS EndpointはEndpointに関係づけられたdata-objectの変更を保存するHistoryCacheを持っている。
これは、RTPS built-in Endpointにも適用される。したがって、それぞれのRTPS built-in Endpointは、
HistoryCacheに書き込まれたデータの論理的な内容を表す DataType に依存する。

図8.30は“DCPSPublication,” “DCPSSubscription,” and “DCPSTopic” TopicsのためのRTPS built-in Endpointに紐付けられたDiscoveredWriterData, DiscoveredReaderData, DiscoveredTopicData DataTypesを定義する。"DCPSParticipant"に紐付けられたDataTypeは8.5.3.2で定義される。

それぞれのRTPS built-in Endpointと関係するDataTypeはDDSによって特定された一致するbuilt-in DDS Entityの情報をすべて保持する。
この理由により、DiscoveredReaderDataはDDS::SubscriptionBuiltinTopicDataを拡張し、DiscoveredWriterDataはDDS::PublicationBuiltinTopicDataを拡張し、
DiscoveredTopicDataはDDS::TopicBuiltinTopicDataを拡張する。

さらに、関連するbuilt-in DDS Entityによって必要とされるデータ、the “Discovered” DataTypesもまた、TPS Endpointを設定するために、プロトコルの実装によって必要とされるすべての情報を含んでいる。この情報は RTPS ReaderProxy, WriterProxyに保存される。

プロトコルの実装はDataTypesに含まれるすべての情報を送信する必要はない。もし一つも情報が存在しなければ、実装はPSMで定義されるデフォルトの値を仮定することができる。

SEDPによって使用されるbuilt-in Endpointとそれらに関連付けられたDataTypesは図 8.31に示される。

## 8.5.5 Interaction with the RTPS virtual machine
SPDPとSEDPについて更に付け加えると、この章ではSPDPによって提供された情報がどのようにRTPS virtual machineのSEDP built-in Endpointsを設定するのに使われるのかを説明する。

## 8.5.5.1 Discovery of a new remote Participant
Using the SPDPbuiltinParticipantReader, a local Participant ‘local_participant’ discovers the existence of another Participant described by the DiscoveredParticipantData participant_data. 
discovereされたParticipantはSEDPを使用する.

以下の疑似コードはdiscovered Participantにある一致するSEDP built-in Endpointsとコミュニケーションするためにlocal SEDP built-in Endpoints within local_participantを設定する。

どのようにEndpointが設定するかはプロトコルの実装に依存する。stateful refarence 実装では、この操作は以下のようなlogical stepsで行われる。
```
// discoverされたparticipantのdomainIdが自分自身のdomainIdと一致するか確認
// もし一致しなければ、local endpointsはdiscoverされたparticipantとコミュニケートするように設定されない。
IF ( participant_data.domainId != local_participant.domainId ) THEN
    RETURN;
ENDIF
// discoverされたparticipantのdomainTagが自分自身のdomainTagと一致するか確認
// もし一致しなければ、local endpointsはdiscoverされたparticipantとコミュニケートするように設定されない。
IF ( !STRING_EQUAL(participant_data.domainTag, local_participant.domainTag) ) THEN
    RETURN;
ENDIF

IF ( PUBLICATIONS_DETECTOR IS_IN participant_data.availableEndpoints ) THEN
    guid = <participant_data.guidPrefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR>;
    writer = local_participant.SEDPbuiltinPublicationsWriter;
    proxy = new ReaderProxy( guid,
    participant_data.metatrafficUnicastLocatorList,
    participant_data.metatrafficMulticastLocatorList);
    writer.matched_reader_add(proxy);
ENDIF

IF ( PUBLICATIONS_ANNOUNCER IS_IN participant_data.availableEndpoints ) THEN
    guid = <participant_data.guidPrefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER>;
    reader = local_participant.SEDPbuiltinPublicationsReader;
    proxy = new WriterProxy( guid,
    participant_data.metatrafficUnicastLocatorList,
    participant_data.metatrafficMulticastLocatorList);
    reader.matched_writer_add(proxy);
ENDIF

IF ( SUBSCRIPTIONS_DETECTOR IS_IN participant_data.availableEndpoints ) THEN
    guid = <participant_data.guidPrefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR>;
    writer = local_participant.SEDPbuiltinSubscriptionsWriter;
    proxy = new ReaderProxy( guid,
    participant_data.metatrafficUnicastLocatorList,
    participant_data.metatrafficMulticastLocatorList);
    writer.matched_reader_add(proxy);
ENDIF

IF ( SUBSCRIPTIONS_ANNOUNCER IS_IN participant_data.availableEndpoints ) THEN
    guid = <participant_data.guidPrefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER>;
    reader = local_participant.SEDPbuiltinSubscriptionsReader;
    proxy = new WriterProxy( guid,
    participant_data.metatrafficUnicastLocatorList,
    participant_data.metatrafficMulticastLocatorList);
    reader.matched_writer_add(proxy);
ENDIF

IF ( TOPICS_DETECTOR IS_IN participant_data.availableEndpoints ) THEN
    guid = <participant_data.guidPrefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR>;
    writer = local_participant.SEDPbuiltinTopicsWriter;
    proxy = new ReaderProxy( guid,
    participant_data.metatrafficUnicastLocatorList,
    participant_data.metatrafficMulticastLocatorList);
    writer.matched_reader_add(proxy);
ENDIF

IF ( TOPICS_ANNOUNCER IS_IN participant_data.availableEndpoints ) THEN
    guid = <participant_data.guidPrefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER>;
    reader = local_participant.SEDPbuiltinTopicsReader;
    proxy = new WriterProxy( guid,
    participant_data.metatrafficUnicastLocatorList,
    participant_data.metatrafficMulticastLocatorList);
    reader.matched_writer_add(proxy);
ENDIF
```

## 8.5.5.2 Removal of a previously discovered Participant
remote ParticipantのleaseDurationに基づき、local Participant ‘local_participant’ は以前発見したGUID_t participant_guidをもつParticipantはこれ以上あらわれないと結論づける。Participant ‘local_participant’は、GUID_t participant_guidによって識別されるParticipant内のエンドポイントと通信していた任意のローカルエンドポイントを再設定しなければなりません。
