use serde_repr::{Deserialize_repr, Serialize_repr};

/// Error codes for Binance.
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq)]
#[repr(i16)]
pub enum ErrorCode {
    // 10xx - General Server or Network issues
    /// An unknown error occurred while processing the request.
    Unknown = -1000,
    /// Internal error; unable to process your request. Please try again.
    Disconnected = -1001,
    /// You are not authorized to execute this request.
    Unauthorized = -1002,
    /// Too many requests queued.
    /// Too much request weight used; current limit is %s request weight per %s. Please use WebSocket Streams for live updates to avoid polling the API.
    /// Way too much request weight used; IP banned until %s. Please use WebSocket Streams for live updates to avoid bans.
    TooManyRequests = -1003,
    /// An unexpected response was received from the message bus. Execution status unknown.
    UnexpectedResp = -1006,
    /// Timeout waiting for response from backend server. Send status unknown; execution status unknown.
    Timeout = -1007,
    /// Server is currently overloaded with other requests. Please try again in a few minutes.
    ServerBusy = -1008,
    /// This code is sent when an error has been returned by the matching engine.
    ErrorMsgReceived = -1010,
    /// The request is rejected by the API. (i.e. The request didn't reach the Matching Engine.)
    /// Potential error messages can be found in Filter Failures or Failures during order placement.
    InvalidMessage = -1013,
    /// Unsupported order combination.
    UnknownOrderComposition = -1014,
    /// Too many new orders.
    /// Too many new orders; current limit is %s orders per %s.
    TooManyOrders = -1015,
    /// This service is no longer available.
    ServiceShuttingDown = -1016,
    /// This operation is not supported.
    UnsupportedOperation = -1020,
    /// Timestamp for this request is outside of the recvWindow.
    /// Timestamp for this request was 1000ms ahead of the server's time.
    InvalidTimestamp = -1021,
    /// Signature for this request is not valid.
    InvalidSignature = -1022,
    /// SenderCompId(49) is currently in use. Concurrent use of the same SenderCompId within one account is not allowed.
    CompIdInUse = -1033,
    /// Too many concurrent connections; current limit is '%s'.
    /// Too many connection attempts for account; current limit is %s per '%s'.
    /// Too many connection attempts from IP; current limit is %s per '%s'.
    TooManyConnections = -1034,
    /// Please send Logout<5> message to close the session.
    LoggedOut = -1035,

    // 11xx - Request issues
    /// Illegal characters found in a parameter.
    /// Illegal characters found in parameter '%s'; legal range is '%s'.
    IllegalChars = -1100,
    /// Too many parameters sent for this endpoint.
    /// Too many parameters; expected '%s' and received '%s'.
    /// Duplicate values for a parameter detected.
    TooManyParameters = -1101,
    /// A mandatory parameter was not sent, was empty/null, or malformed.
    /// Mandatory parameter '%s' was not sent, was empty/null, or malformed.
    /// Param '%s' or '%s' must be sent, but both were empty/null!
    /// Required tag '%s' missing.
    /// Field value was empty or malformed.
    /// '%s' contains unexpected value. Cannot be greater than %s.
    MandatoryParamEmptyOrMalformed = -1102,
    /// An unknown parameter was sent.
    /// Undefined Tag.
    UnknownParam = -1103,
    /// Not all sent parameters were read.
    /// Not all sent parameters were read; read '%s' parameter(s) but was sent '%s'.
    UnreadParameters = -1104,
    /// A parameter was empty.
    /// Parameter '%s' was empty.
    ParamEmpty = -1105,
    /// A parameter was sent when not required.
    /// Parameter '%s' sent when not required.
    /// A tag '%s' was sent when not required.
    ParamNotRequired = -1106,
    /// Parameter '%s' overflowed.
    ParamOverflow = -1108,
    /// Parameter '%s' has too much precision.
    BadPrecision = -1111,
    /// No orders on book for symbol.
    NoDepth = -1112,
    /// TimeInForce parameter sent when not required.
    TifNotRequired = -1114,
    /// Invalid timeInForce.
    InvalidTif = -1115,
    /// Invalid orderType.
    InvalidOrderType = -1116,
    /// Invalid side.
    InvalidSide = -1117,
    /// New client order ID was empty.
    EmptyNewClOrdId = -1118,
    /// Original client order ID was empty.
    EmptyOrgClOrdId = -1119,
    /// Invalid interval.
    BadInterval = -1120,
    /// Invalid symbol.
    BadSymbol = -1121,
    /// Invalid symbolStatus.
    InvalidSymbolstatus = -1122,
    /// This listenKey does not exist.
    InvalidListenKey = -1125,
    /// Lookup interval is too big.
    /// More than %s hours between startTime and endTime.
    MoreThanXxHours = -1127,
    /// Combination of optional parameters invalid.
    /// Combination of optional fields invalid. Recommendation: '%s' and '%s' must both be sent.
    /// Fields [%s] must be sent together or omitted entirely.
    /// Invalid MDEntryType (269) combination. BID and OFFER must be requested together.
    OptionalParamsBadCombo = -1128,
    /// Invalid data sent for a parameter.
    /// Data sent for parameter '%s' is not valid.
    InvalidParameter = -1130,
    /// strategyType was less than 1000000.
    /// TargetStrategy (847) was less than 1000000.
    BadStrategyType = -1134,
    /// Invalid JSON Request
    /// JSON sent for parameter '%s' is not valid
    InvalidJson = -1135,
    /// Invalid ticker type.
    InvalidTickerType = -1139,
    /// cancelRestrictions has to be either ONLY_NEW or ONLY_PARTIALLY_FILLED.
    InvalidCancelRestrictions = -1145,
    /// Symbol is present multiple times in the list.
    DuplicateSymbols = -1151,
    /// Invalid X-MBX-SBE header; expected <SCHEMA_ID>:<VERSION>.
    InvalidSbeHeader = -1152,
    /// Unsupported SBE schema ID or version specified in the X-MBX-SBE header.
    UnsupportedSchemaId = -1153,
    /// SBE is not enabled.
    SbeDisabled = -1155,
    /// Order type not supported in OCO.
    /// If the order type provided in the aboveType and/or belowType is not supported.
    OcoOrderTypeRejected = -1158,
    /// Parameter '%s' is not supported if aboveTimeInForce/belowTimeInForce is not GTC.
    /// If the order type for the above or below leg is STOP_LOSS_LIMIT, and icebergQty is provided for that leg, the timeInForce has to be GTC else it will throw an error.
    /// TimeInForce (59) must be GTC (1) when MaxFloor (111) is used.
    OcoIcebergqtyTimeinforce = -1160,
    /// Unable to encode the response in SBE schema 'x'. Please use schema 'y' or higher.
    DeprecatedSchema = -1161,
    /// A limit order in a buy OCO must be below.
    BuyOcoLimitMustBeBelow = -1165,
    /// A limit order in a sell OCO must be above.
    SellOcoLimitMustBeAbove = -1166,
    /// At least one OCO order must be contingent.
    BothOcoOrdersCannotBeLimit = -1168,
    /// Invalid tag number.
    InvalidTagNumber = -1169,
    /// Tag '%s' not defined for this message type.
    TagNotDefinedInMessage = -1170,
    /// Tag '%s' appears more than once.
    TagAppearsMoreThanOnce = -1171,
    /// Tag '%s' specified out of required order.
    TagOutOfOrder = -1172,
    /// Repeating group '%s' fields out of order.
    GroupFieldsOutOfOrder = -1173,
    /// Component '%s' is incorrectly populated on '%s' order. Recommendation: '%s'
    InvalidComponent = -1174,
    /// Continuation of sequence numbers to new session is currently unsupported. Sequence numbers must be reset for each new session.
    ResetSeqNumSupport = -1175,
    /// Logon<A> should only be sent once.
    AlreadyLoggedIn = -1176,
    /// CheckSum(10) contains an incorrect value.
    /// BeginString (8) is not the first tag in a message.
    /// MsgType (35) is not the third tag in a message.
    /// BodyLength (9) does not contain the correct byte count.
    /// Only printable ASCII characters and SOH (Start of Header) are allowed.
    GarbledMessage = -1177,
    /// SenderCompId(49) contains an incorrect value. The SenderCompID value should not change throughout the lifetime of a session.
    BadSenderCompid = -1178,
    /// MsgSeqNum(34) contains an unexpected value. Expected: '%d'.
    BadSeqNum = -1179,
    /// Logon<A> must be the first message in the session.
    ExpectedLogon = -1180,
    /// Too many messages; current limit is '%d' messages per '%s'.
    TooManyMessages = -1181,
    /// Conflicting fields: [%s]
    ParamsBadCombo = -1182,
    /// Requested operation is not allowed in DropCopy sessions.
    NotAllowedInDropCopySessions = -1183,
    /// DropCopy sessions are not supported on this server. Please reconnect to a drop copy server.
    DropCopySessionNotAllowed = -1184,
    /// Only DropCopy sessions are supported on this server. Either reconnect to order entry server or send DropCopyFlag (9406) field.
    DropCopySessionRequired = -1185,
    /// Requested operation is not allowed in order entry sessions.
    NotAllowedInOrderEntrySessions = -1186,
    /// Requested operation is not allowed in market data sessions.
    NotAllowedInMarketDataSessions = -1187,
    /// Incorrect NumInGroup count for repeating group '%s'.
    IncorrectNumInGroupCount = -1188,
    /// Group '%s' contains duplicate entries.
    DuplicateEntriesInAGroup = -1189,
    /// MDReqID (262) contains a subscription request id that is already in use on this connection.
    /// MDReqID (262) contains an unsubscription request id that does not match any active subscription.
    InvalidRequestId = -1190,
    /// Too many subscriptions. Connection may create up to '%s' subscriptions at a time.
    /// Similar subscription is already active on this connection. Symbol='%s', active subscription id: '%s'.
    TooManySubscriptions = -1191,
    /// Invalid value for time unit; expected either MICROSECOND or MILLISECOND.
    InvalidTimeUnit = -1194,
    /// A stop loss order in a buy OCO must be above.
    BuyOcoStopLossMustBeAbove = -1196,
    /// A stop loss order in a sell OCO must be below.
    SellOcoStopLossMustBeBelow = -1197,
    /// A take profit order in a buy OCO must be below.
    BuyOcoTakeProfitMustBeBelow = -1198,
    /// A take profit order in a sell OCO must be above.
    SellOcoTakeProfitMustBeAbove = -1199,
    /// NEW_ORDER_REJECTED
    /// This code is sent when an error has been returned by the matching engine.
    NewOrderRejected = -2010,
    /// CANCEL_REJECTED
    /// This code is sent when an error has been returned by the matching engine.
    CancelRejected = -2011,
    /// Order does not exist.
    NoSuchOrder = -2013,
    /// API-key format invalid.
    BadApiKeyFmt = -2014,
    /// Invalid API-key, IP, or permissions for action.
    RejectedMbxKey = -2015,
    /// No trading window could be found for the symbol. Try ticker/24hrs instead.
    NoTradingWindow = -2016,
    /// This code is sent when either the cancellation of the order failed or the new order placement failed but not both.
    /// Errors regarding placing orders via cancelReplace
    OrderCancelReplacePartiallyFailed = -2021,
    /// This code is sent when both the cancellation of the order failed and the new order placement failed.
    /// Errors regarding placing orders via cancelReplace
    OrderCancelReplaceFailed = -2022,
    /// Order was canceled or expired with no executed qty over 90 days ago and has been archived.
    OrderArchived = -2026,
    /// This code is sent when an error has been returned by the matching engine.
    OrderAmendRejected = -2038,
    /// Client order ID is not correct for this order ID.
    ClientOrderIdInvalid = -2039,
}
