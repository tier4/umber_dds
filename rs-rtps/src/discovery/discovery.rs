// SPDPbuiltinParticipantWriter
// RTPS Best-Effort StatelessWriter
// HistoryCacheには
// a single data-object of type SPDPdiscoveredParticipantDataを含む
// data-objectを、事前に設定されたlocatorのリストにParticipantの存在を知らせるためにnetworkに送信する。
// このとき、HistoryCacheに存在するすべてのchangesをすべてのlocatorに送信するStatelessWriter::unsent_changes_resetを定期的に呼び出すことで達成される。
// 事前に設定されたlocatorのリストにはSPDP_WELL_KNOWN_UNICAST_PORT,
// SPDP_WELL_KNOWN_MULTICAST_PORTのポートを使わなければならない。
//
// SPDPbuiltinParticipantReader
// RTPS Reader
// remote ParticipantからSPDPdiscoveredParticipantData announcementを受信する。

pub struct Discovery {}
