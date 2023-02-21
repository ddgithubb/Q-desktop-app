/* eslint-disable */
import Long from "long";
import _m0 from "protobufjs/minimal";

export const protobufPackage = "pool.v1";

export enum PoolMediaType {
  IMAGE = 0,
  UNRECOGNIZED = -1,
}

export function poolMediaTypeFromJSON(object: any): PoolMediaType {
  switch (object) {
    case 0:
    case "IMAGE":
      return PoolMediaType.IMAGE;
    case -1:
    case "UNRECOGNIZED":
    default:
      return PoolMediaType.UNRECOGNIZED;
  }
}

export function poolMediaTypeToJSON(object: PoolMediaType): string {
  switch (object) {
    case PoolMediaType.IMAGE:
      return "IMAGE";
    case PoolMediaType.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

export interface PoolFileInfo {
  fileId: string;
  fileName: string;
  totalSize: number;
  originNodeId: string;
}

export interface PoolFileSeeders {
  fileInfo: PoolFileInfo | undefined;
  seederNodeIds: string[];
}

export interface PoolImageData {
  width: number;
  height: number;
  previewImageBase64: string;
}

export interface PoolChunkRange {
  /** inclusive */
  start: number;
  /** inclusive */
  end: number;
}

export interface PoolMessage {
  msgId: string;
  type: PoolMessage_Type;
  userId: string;
  created: number;
  data?:
    | { $case: "nodeInfoData"; nodeInfoData: PoolMessage_NodeInfoData }
    | { $case: "textData"; textData: PoolMessage_TextData }
    | { $case: "fileOfferData"; fileOfferData: PoolFileInfo }
    | { $case: "mediaOfferData"; mediaOfferData: PoolMessage_MediaOfferData }
    | { $case: "fileRequestData"; fileRequestData: PoolMessage_FileRequestData }
    | { $case: "retractFileOfferData"; retractFileOfferData: PoolMessage_RetractFileOfferData }
    | { $case: "retractFileRequestData"; retractFileRequestData: PoolMessage_RetractFileRequestData };
}

export enum PoolMessage_Type {
  NODE_INFO = 0,
  TEXT = 1,
  FILE_OFFER = 2,
  MEDIA_OFFER = 3,
  FILE_REQUEST = 4,
  RETRACT_FILE_OFFER = 5,
  RETRACT_FILE_REQUEST = 6,
  UNRECOGNIZED = -1,
}

export function poolMessage_TypeFromJSON(object: any): PoolMessage_Type {
  switch (object) {
    case 0:
    case "NODE_INFO":
      return PoolMessage_Type.NODE_INFO;
    case 1:
    case "TEXT":
      return PoolMessage_Type.TEXT;
    case 2:
    case "FILE_OFFER":
      return PoolMessage_Type.FILE_OFFER;
    case 3:
    case "MEDIA_OFFER":
      return PoolMessage_Type.MEDIA_OFFER;
    case 4:
    case "FILE_REQUEST":
      return PoolMessage_Type.FILE_REQUEST;
    case 5:
    case "RETRACT_FILE_OFFER":
      return PoolMessage_Type.RETRACT_FILE_OFFER;
    case 6:
    case "RETRACT_FILE_REQUEST":
      return PoolMessage_Type.RETRACT_FILE_REQUEST;
    case -1:
    case "UNRECOGNIZED":
    default:
      return PoolMessage_Type.UNRECOGNIZED;
  }
}

export function poolMessage_TypeToJSON(object: PoolMessage_Type): string {
  switch (object) {
    case PoolMessage_Type.NODE_INFO:
      return "NODE_INFO";
    case PoolMessage_Type.TEXT:
      return "TEXT";
    case PoolMessage_Type.FILE_OFFER:
      return "FILE_OFFER";
    case PoolMessage_Type.MEDIA_OFFER:
      return "MEDIA_OFFER";
    case PoolMessage_Type.FILE_REQUEST:
      return "FILE_REQUEST";
    case PoolMessage_Type.RETRACT_FILE_OFFER:
      return "RETRACT_FILE_OFFER";
    case PoolMessage_Type.RETRACT_FILE_REQUEST:
      return "RETRACT_FILE_REQUEST";
    case PoolMessage_Type.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

export interface PoolMessage_NodeInfoData {
  /** append only */
  fileOffers: PoolFileInfo[];
}

export interface PoolMessage_TextData {
  text: string;
}

export interface PoolMessage_MediaOfferData {
  fileInfo: PoolFileInfo | undefined;
  mediaType: PoolMediaType;
  mediaData?: { $case: "imageData"; imageData: PoolImageData };
}

export interface PoolMessage_FileRequestData {
  fileId: string;
  requestedChunks: PoolChunkRange[];
  promisedChunks: PoolChunkRange[];
  requestFromOrigin: boolean;
}

export interface PoolMessage_RetractFileOfferData {
  fileId: string;
}

export interface PoolMessage_RetractFileRequestData {
  fileId: string;
}

export interface PoolDirectMessage {
  type: PoolDirectMessage_DirectType;
  data?: { $case: "latestReplyData"; latestReplyData: PoolDirectMessage_LatestReplyData };
}

export enum PoolDirectMessage_DirectType {
  LATEST_REQUEST = 0,
  LATEST_REPLY = 1,
  UNRECOGNIZED = -1,
}

export function poolDirectMessage_DirectTypeFromJSON(object: any): PoolDirectMessage_DirectType {
  switch (object) {
    case 0:
    case "LATEST_REQUEST":
      return PoolDirectMessage_DirectType.LATEST_REQUEST;
    case 1:
    case "LATEST_REPLY":
      return PoolDirectMessage_DirectType.LATEST_REPLY;
    case -1:
    case "UNRECOGNIZED":
    default:
      return PoolDirectMessage_DirectType.UNRECOGNIZED;
  }
}

export function poolDirectMessage_DirectTypeToJSON(object: PoolDirectMessage_DirectType): string {
  switch (object) {
    case PoolDirectMessage_DirectType.LATEST_REQUEST:
      return "LATEST_REQUEST";
    case PoolDirectMessage_DirectType.LATEST_REPLY:
      return "LATEST_REPLY";
    case PoolDirectMessage_DirectType.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

export interface PoolDirectMessage_LatestReplyData {
  latestMessages: PoolMessage[];
  fileSeeders: PoolFileSeeders[];
}

export interface PoolMessagePackageSourceInfo {
  nodeId: string;
  path: number[];
}

export interface PoolMessagePackageDestinationInfo {
  nodeId: string;
}

export interface PoolChunkMessage {
  fileId: string;
  chunkNumber: number;
  chunk: Uint8Array;
}

export interface PoolMessagePackage {
  src: PoolMessagePackageSourceInfo | undefined;
  dests: PoolMessagePackageDestinationInfo[];
  partnerIntPath?: number | undefined;
  msg?: PoolMessage | undefined;
  directMsg?: PoolDirectMessage | undefined;
  chunkMsg?: PoolChunkMessage | undefined;
}

function createBasePoolFileInfo(): PoolFileInfo {
  return { fileId: "", fileName: "", totalSize: 0, originNodeId: "" };
}

export const PoolFileInfo = {
  encode(message: PoolFileInfo, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.fileId !== "") {
      writer.uint32(10).string(message.fileId);
    }
    if (message.fileName !== "") {
      writer.uint32(18).string(message.fileName);
    }
    if (message.totalSize !== 0) {
      writer.uint32(24).uint64(message.totalSize);
    }
    if (message.originNodeId !== "") {
      writer.uint32(34).string(message.originNodeId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolFileInfo {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolFileInfo();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.fileId = reader.string();
          break;
        case 2:
          message.fileName = reader.string();
          break;
        case 3:
          message.totalSize = longToNumber(reader.uint64() as Long);
          break;
        case 4:
          message.originNodeId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolFileInfo {
    return {
      fileId: isSet(object.fileId) ? String(object.fileId) : "",
      fileName: isSet(object.fileName) ? String(object.fileName) : "",
      totalSize: isSet(object.totalSize) ? Number(object.totalSize) : 0,
      originNodeId: isSet(object.originNodeId) ? String(object.originNodeId) : "",
    };
  },

  toJSON(message: PoolFileInfo): unknown {
    const obj: any = {};
    message.fileId !== undefined && (obj.fileId = message.fileId);
    message.fileName !== undefined && (obj.fileName = message.fileName);
    message.totalSize !== undefined && (obj.totalSize = Math.round(message.totalSize));
    message.originNodeId !== undefined && (obj.originNodeId = message.originNodeId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolFileInfo>, I>>(object: I): PoolFileInfo {
    const message = createBasePoolFileInfo();
    message.fileId = object.fileId ?? "";
    message.fileName = object.fileName ?? "";
    message.totalSize = object.totalSize ?? 0;
    message.originNodeId = object.originNodeId ?? "";
    return message;
  },
};

function createBasePoolFileSeeders(): PoolFileSeeders {
  return { fileInfo: undefined, seederNodeIds: [] };
}

export const PoolFileSeeders = {
  encode(message: PoolFileSeeders, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.fileInfo !== undefined) {
      PoolFileInfo.encode(message.fileInfo, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.seederNodeIds) {
      writer.uint32(18).string(v!);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolFileSeeders {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolFileSeeders();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.fileInfo = PoolFileInfo.decode(reader, reader.uint32());
          break;
        case 2:
          message.seederNodeIds.push(reader.string());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolFileSeeders {
    return {
      fileInfo: isSet(object.fileInfo) ? PoolFileInfo.fromJSON(object.fileInfo) : undefined,
      seederNodeIds: Array.isArray(object?.seederNodeIds) ? object.seederNodeIds.map((e: any) => String(e)) : [],
    };
  },

  toJSON(message: PoolFileSeeders): unknown {
    const obj: any = {};
    message.fileInfo !== undefined &&
      (obj.fileInfo = message.fileInfo ? PoolFileInfo.toJSON(message.fileInfo) : undefined);
    if (message.seederNodeIds) {
      obj.seederNodeIds = message.seederNodeIds.map((e) => e);
    } else {
      obj.seederNodeIds = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolFileSeeders>, I>>(object: I): PoolFileSeeders {
    const message = createBasePoolFileSeeders();
    message.fileInfo = (object.fileInfo !== undefined && object.fileInfo !== null)
      ? PoolFileInfo.fromPartial(object.fileInfo)
      : undefined;
    message.seederNodeIds = object.seederNodeIds?.map((e) => e) || [];
    return message;
  },
};

function createBasePoolImageData(): PoolImageData {
  return { width: 0, height: 0, previewImageBase64: "" };
}

export const PoolImageData = {
  encode(message: PoolImageData, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.width !== 0) {
      writer.uint32(8).uint32(message.width);
    }
    if (message.height !== 0) {
      writer.uint32(16).uint32(message.height);
    }
    if (message.previewImageBase64 !== "") {
      writer.uint32(26).string(message.previewImageBase64);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolImageData {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolImageData();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.width = reader.uint32();
          break;
        case 2:
          message.height = reader.uint32();
          break;
        case 3:
          message.previewImageBase64 = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolImageData {
    return {
      width: isSet(object.width) ? Number(object.width) : 0,
      height: isSet(object.height) ? Number(object.height) : 0,
      previewImageBase64: isSet(object.previewImageBase64) ? String(object.previewImageBase64) : "",
    };
  },

  toJSON(message: PoolImageData): unknown {
    const obj: any = {};
    message.width !== undefined && (obj.width = Math.round(message.width));
    message.height !== undefined && (obj.height = Math.round(message.height));
    message.previewImageBase64 !== undefined && (obj.previewImageBase64 = message.previewImageBase64);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolImageData>, I>>(object: I): PoolImageData {
    const message = createBasePoolImageData();
    message.width = object.width ?? 0;
    message.height = object.height ?? 0;
    message.previewImageBase64 = object.previewImageBase64 ?? "";
    return message;
  },
};

function createBasePoolChunkRange(): PoolChunkRange {
  return { start: 0, end: 0 };
}

export const PoolChunkRange = {
  encode(message: PoolChunkRange, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.start !== 0) {
      writer.uint32(8).uint64(message.start);
    }
    if (message.end !== 0) {
      writer.uint32(16).uint64(message.end);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolChunkRange {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolChunkRange();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.start = longToNumber(reader.uint64() as Long);
          break;
        case 2:
          message.end = longToNumber(reader.uint64() as Long);
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolChunkRange {
    return { start: isSet(object.start) ? Number(object.start) : 0, end: isSet(object.end) ? Number(object.end) : 0 };
  },

  toJSON(message: PoolChunkRange): unknown {
    const obj: any = {};
    message.start !== undefined && (obj.start = Math.round(message.start));
    message.end !== undefined && (obj.end = Math.round(message.end));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolChunkRange>, I>>(object: I): PoolChunkRange {
    const message = createBasePoolChunkRange();
    message.start = object.start ?? 0;
    message.end = object.end ?? 0;
    return message;
  },
};

function createBasePoolMessage(): PoolMessage {
  return { msgId: "", type: 0, userId: "", created: 0, data: undefined };
}

export const PoolMessage = {
  encode(message: PoolMessage, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.msgId !== "") {
      writer.uint32(10).string(message.msgId);
    }
    if (message.type !== 0) {
      writer.uint32(16).int32(message.type);
    }
    if (message.userId !== "") {
      writer.uint32(26).string(message.userId);
    }
    if (message.created !== 0) {
      writer.uint32(32).uint64(message.created);
    }
    if (message.data?.$case === "nodeInfoData") {
      PoolMessage_NodeInfoData.encode(message.data.nodeInfoData, writer.uint32(42).fork()).ldelim();
    }
    if (message.data?.$case === "textData") {
      PoolMessage_TextData.encode(message.data.textData, writer.uint32(50).fork()).ldelim();
    }
    if (message.data?.$case === "fileOfferData") {
      PoolFileInfo.encode(message.data.fileOfferData, writer.uint32(58).fork()).ldelim();
    }
    if (message.data?.$case === "mediaOfferData") {
      PoolMessage_MediaOfferData.encode(message.data.mediaOfferData, writer.uint32(66).fork()).ldelim();
    }
    if (message.data?.$case === "fileRequestData") {
      PoolMessage_FileRequestData.encode(message.data.fileRequestData, writer.uint32(74).fork()).ldelim();
    }
    if (message.data?.$case === "retractFileOfferData") {
      PoolMessage_RetractFileOfferData.encode(message.data.retractFileOfferData, writer.uint32(82).fork()).ldelim();
    }
    if (message.data?.$case === "retractFileRequestData") {
      PoolMessage_RetractFileRequestData.encode(message.data.retractFileRequestData, writer.uint32(90).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessage {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessage();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.msgId = reader.string();
          break;
        case 2:
          message.type = reader.int32() as any;
          break;
        case 3:
          message.userId = reader.string();
          break;
        case 4:
          message.created = longToNumber(reader.uint64() as Long);
          break;
        case 5:
          message.data = {
            $case: "nodeInfoData",
            nodeInfoData: PoolMessage_NodeInfoData.decode(reader, reader.uint32()),
          };
          break;
        case 6:
          message.data = { $case: "textData", textData: PoolMessage_TextData.decode(reader, reader.uint32()) };
          break;
        case 7:
          message.data = { $case: "fileOfferData", fileOfferData: PoolFileInfo.decode(reader, reader.uint32()) };
          break;
        case 8:
          message.data = {
            $case: "mediaOfferData",
            mediaOfferData: PoolMessage_MediaOfferData.decode(reader, reader.uint32()),
          };
          break;
        case 9:
          message.data = {
            $case: "fileRequestData",
            fileRequestData: PoolMessage_FileRequestData.decode(reader, reader.uint32()),
          };
          break;
        case 10:
          message.data = {
            $case: "retractFileOfferData",
            retractFileOfferData: PoolMessage_RetractFileOfferData.decode(reader, reader.uint32()),
          };
          break;
        case 11:
          message.data = {
            $case: "retractFileRequestData",
            retractFileRequestData: PoolMessage_RetractFileRequestData.decode(reader, reader.uint32()),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessage {
    return {
      msgId: isSet(object.msgId) ? String(object.msgId) : "",
      type: isSet(object.type) ? poolMessage_TypeFromJSON(object.type) : 0,
      userId: isSet(object.userId) ? String(object.userId) : "",
      created: isSet(object.created) ? Number(object.created) : 0,
      data: isSet(object.nodeInfoData)
        ? { $case: "nodeInfoData", nodeInfoData: PoolMessage_NodeInfoData.fromJSON(object.nodeInfoData) }
        : isSet(object.textData)
        ? { $case: "textData", textData: PoolMessage_TextData.fromJSON(object.textData) }
        : isSet(object.fileOfferData)
        ? { $case: "fileOfferData", fileOfferData: PoolFileInfo.fromJSON(object.fileOfferData) }
        : isSet(object.mediaOfferData)
        ? { $case: "mediaOfferData", mediaOfferData: PoolMessage_MediaOfferData.fromJSON(object.mediaOfferData) }
        : isSet(object.fileRequestData)
        ? { $case: "fileRequestData", fileRequestData: PoolMessage_FileRequestData.fromJSON(object.fileRequestData) }
        : isSet(object.retractFileOfferData)
        ? {
          $case: "retractFileOfferData",
          retractFileOfferData: PoolMessage_RetractFileOfferData.fromJSON(object.retractFileOfferData),
        }
        : isSet(object.retractFileRequestData)
        ? {
          $case: "retractFileRequestData",
          retractFileRequestData: PoolMessage_RetractFileRequestData.fromJSON(object.retractFileRequestData),
        }
        : undefined,
    };
  },

  toJSON(message: PoolMessage): unknown {
    const obj: any = {};
    message.msgId !== undefined && (obj.msgId = message.msgId);
    message.type !== undefined && (obj.type = poolMessage_TypeToJSON(message.type));
    message.userId !== undefined && (obj.userId = message.userId);
    message.created !== undefined && (obj.created = Math.round(message.created));
    message.data?.$case === "nodeInfoData" && (obj.nodeInfoData = message.data?.nodeInfoData
      ? PoolMessage_NodeInfoData.toJSON(message.data?.nodeInfoData)
      : undefined);
    message.data?.$case === "textData" &&
      (obj.textData = message.data?.textData ? PoolMessage_TextData.toJSON(message.data?.textData) : undefined);
    message.data?.$case === "fileOfferData" &&
      (obj.fileOfferData = message.data?.fileOfferData ? PoolFileInfo.toJSON(message.data?.fileOfferData) : undefined);
    message.data?.$case === "mediaOfferData" && (obj.mediaOfferData = message.data?.mediaOfferData
      ? PoolMessage_MediaOfferData.toJSON(message.data?.mediaOfferData)
      : undefined);
    message.data?.$case === "fileRequestData" && (obj.fileRequestData = message.data?.fileRequestData
      ? PoolMessage_FileRequestData.toJSON(message.data?.fileRequestData)
      : undefined);
    message.data?.$case === "retractFileOfferData" && (obj.retractFileOfferData = message.data?.retractFileOfferData
      ? PoolMessage_RetractFileOfferData.toJSON(message.data?.retractFileOfferData)
      : undefined);
    message.data?.$case === "retractFileRequestData" &&
      (obj.retractFileRequestData = message.data?.retractFileRequestData
        ? PoolMessage_RetractFileRequestData.toJSON(message.data?.retractFileRequestData)
        : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessage>, I>>(object: I): PoolMessage {
    const message = createBasePoolMessage();
    message.msgId = object.msgId ?? "";
    message.type = object.type ?? 0;
    message.userId = object.userId ?? "";
    message.created = object.created ?? 0;
    if (
      object.data?.$case === "nodeInfoData" &&
      object.data?.nodeInfoData !== undefined &&
      object.data?.nodeInfoData !== null
    ) {
      message.data = {
        $case: "nodeInfoData",
        nodeInfoData: PoolMessage_NodeInfoData.fromPartial(object.data.nodeInfoData),
      };
    }
    if (object.data?.$case === "textData" && object.data?.textData !== undefined && object.data?.textData !== null) {
      message.data = { $case: "textData", textData: PoolMessage_TextData.fromPartial(object.data.textData) };
    }
    if (
      object.data?.$case === "fileOfferData" &&
      object.data?.fileOfferData !== undefined &&
      object.data?.fileOfferData !== null
    ) {
      message.data = { $case: "fileOfferData", fileOfferData: PoolFileInfo.fromPartial(object.data.fileOfferData) };
    }
    if (
      object.data?.$case === "mediaOfferData" &&
      object.data?.mediaOfferData !== undefined &&
      object.data?.mediaOfferData !== null
    ) {
      message.data = {
        $case: "mediaOfferData",
        mediaOfferData: PoolMessage_MediaOfferData.fromPartial(object.data.mediaOfferData),
      };
    }
    if (
      object.data?.$case === "fileRequestData" &&
      object.data?.fileRequestData !== undefined &&
      object.data?.fileRequestData !== null
    ) {
      message.data = {
        $case: "fileRequestData",
        fileRequestData: PoolMessage_FileRequestData.fromPartial(object.data.fileRequestData),
      };
    }
    if (
      object.data?.$case === "retractFileOfferData" &&
      object.data?.retractFileOfferData !== undefined &&
      object.data?.retractFileOfferData !== null
    ) {
      message.data = {
        $case: "retractFileOfferData",
        retractFileOfferData: PoolMessage_RetractFileOfferData.fromPartial(object.data.retractFileOfferData),
      };
    }
    if (
      object.data?.$case === "retractFileRequestData" &&
      object.data?.retractFileRequestData !== undefined &&
      object.data?.retractFileRequestData !== null
    ) {
      message.data = {
        $case: "retractFileRequestData",
        retractFileRequestData: PoolMessage_RetractFileRequestData.fromPartial(object.data.retractFileRequestData),
      };
    }
    return message;
  },
};

function createBasePoolMessage_NodeInfoData(): PoolMessage_NodeInfoData {
  return { fileOffers: [] };
}

export const PoolMessage_NodeInfoData = {
  encode(message: PoolMessage_NodeInfoData, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.fileOffers) {
      PoolFileInfo.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessage_NodeInfoData {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessage_NodeInfoData();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.fileOffers.push(PoolFileInfo.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessage_NodeInfoData {
    return {
      fileOffers: Array.isArray(object?.fileOffers) ? object.fileOffers.map((e: any) => PoolFileInfo.fromJSON(e)) : [],
    };
  },

  toJSON(message: PoolMessage_NodeInfoData): unknown {
    const obj: any = {};
    if (message.fileOffers) {
      obj.fileOffers = message.fileOffers.map((e) => e ? PoolFileInfo.toJSON(e) : undefined);
    } else {
      obj.fileOffers = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessage_NodeInfoData>, I>>(object: I): PoolMessage_NodeInfoData {
    const message = createBasePoolMessage_NodeInfoData();
    message.fileOffers = object.fileOffers?.map((e) => PoolFileInfo.fromPartial(e)) || [];
    return message;
  },
};

function createBasePoolMessage_TextData(): PoolMessage_TextData {
  return { text: "" };
}

export const PoolMessage_TextData = {
  encode(message: PoolMessage_TextData, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.text !== "") {
      writer.uint32(10).string(message.text);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessage_TextData {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessage_TextData();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.text = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessage_TextData {
    return { text: isSet(object.text) ? String(object.text) : "" };
  },

  toJSON(message: PoolMessage_TextData): unknown {
    const obj: any = {};
    message.text !== undefined && (obj.text = message.text);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessage_TextData>, I>>(object: I): PoolMessage_TextData {
    const message = createBasePoolMessage_TextData();
    message.text = object.text ?? "";
    return message;
  },
};

function createBasePoolMessage_MediaOfferData(): PoolMessage_MediaOfferData {
  return { fileInfo: undefined, mediaType: 0, mediaData: undefined };
}

export const PoolMessage_MediaOfferData = {
  encode(message: PoolMessage_MediaOfferData, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.fileInfo !== undefined) {
      PoolFileInfo.encode(message.fileInfo, writer.uint32(10).fork()).ldelim();
    }
    if (message.mediaType !== 0) {
      writer.uint32(16).int32(message.mediaType);
    }
    if (message.mediaData?.$case === "imageData") {
      PoolImageData.encode(message.mediaData.imageData, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessage_MediaOfferData {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessage_MediaOfferData();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.fileInfo = PoolFileInfo.decode(reader, reader.uint32());
          break;
        case 2:
          message.mediaType = reader.int32() as any;
          break;
        case 3:
          message.mediaData = { $case: "imageData", imageData: PoolImageData.decode(reader, reader.uint32()) };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessage_MediaOfferData {
    return {
      fileInfo: isSet(object.fileInfo) ? PoolFileInfo.fromJSON(object.fileInfo) : undefined,
      mediaType: isSet(object.mediaType) ? poolMediaTypeFromJSON(object.mediaType) : 0,
      mediaData: isSet(object.imageData)
        ? { $case: "imageData", imageData: PoolImageData.fromJSON(object.imageData) }
        : undefined,
    };
  },

  toJSON(message: PoolMessage_MediaOfferData): unknown {
    const obj: any = {};
    message.fileInfo !== undefined &&
      (obj.fileInfo = message.fileInfo ? PoolFileInfo.toJSON(message.fileInfo) : undefined);
    message.mediaType !== undefined && (obj.mediaType = poolMediaTypeToJSON(message.mediaType));
    message.mediaData?.$case === "imageData" &&
      (obj.imageData = message.mediaData?.imageData ? PoolImageData.toJSON(message.mediaData?.imageData) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessage_MediaOfferData>, I>>(object: I): PoolMessage_MediaOfferData {
    const message = createBasePoolMessage_MediaOfferData();
    message.fileInfo = (object.fileInfo !== undefined && object.fileInfo !== null)
      ? PoolFileInfo.fromPartial(object.fileInfo)
      : undefined;
    message.mediaType = object.mediaType ?? 0;
    if (
      object.mediaData?.$case === "imageData" &&
      object.mediaData?.imageData !== undefined &&
      object.mediaData?.imageData !== null
    ) {
      message.mediaData = { $case: "imageData", imageData: PoolImageData.fromPartial(object.mediaData.imageData) };
    }
    return message;
  },
};

function createBasePoolMessage_FileRequestData(): PoolMessage_FileRequestData {
  return { fileId: "", requestedChunks: [], promisedChunks: [], requestFromOrigin: false };
}

export const PoolMessage_FileRequestData = {
  encode(message: PoolMessage_FileRequestData, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.fileId !== "") {
      writer.uint32(10).string(message.fileId);
    }
    for (const v of message.requestedChunks) {
      PoolChunkRange.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    for (const v of message.promisedChunks) {
      PoolChunkRange.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    if (message.requestFromOrigin === true) {
      writer.uint32(32).bool(message.requestFromOrigin);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessage_FileRequestData {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessage_FileRequestData();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.fileId = reader.string();
          break;
        case 2:
          message.requestedChunks.push(PoolChunkRange.decode(reader, reader.uint32()));
          break;
        case 3:
          message.promisedChunks.push(PoolChunkRange.decode(reader, reader.uint32()));
          break;
        case 4:
          message.requestFromOrigin = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessage_FileRequestData {
    return {
      fileId: isSet(object.fileId) ? String(object.fileId) : "",
      requestedChunks: Array.isArray(object?.requestedChunks)
        ? object.requestedChunks.map((e: any) => PoolChunkRange.fromJSON(e))
        : [],
      promisedChunks: Array.isArray(object?.promisedChunks)
        ? object.promisedChunks.map((e: any) => PoolChunkRange.fromJSON(e))
        : [],
      requestFromOrigin: isSet(object.requestFromOrigin) ? Boolean(object.requestFromOrigin) : false,
    };
  },

  toJSON(message: PoolMessage_FileRequestData): unknown {
    const obj: any = {};
    message.fileId !== undefined && (obj.fileId = message.fileId);
    if (message.requestedChunks) {
      obj.requestedChunks = message.requestedChunks.map((e) => e ? PoolChunkRange.toJSON(e) : undefined);
    } else {
      obj.requestedChunks = [];
    }
    if (message.promisedChunks) {
      obj.promisedChunks = message.promisedChunks.map((e) => e ? PoolChunkRange.toJSON(e) : undefined);
    } else {
      obj.promisedChunks = [];
    }
    message.requestFromOrigin !== undefined && (obj.requestFromOrigin = message.requestFromOrigin);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessage_FileRequestData>, I>>(object: I): PoolMessage_FileRequestData {
    const message = createBasePoolMessage_FileRequestData();
    message.fileId = object.fileId ?? "";
    message.requestedChunks = object.requestedChunks?.map((e) => PoolChunkRange.fromPartial(e)) || [];
    message.promisedChunks = object.promisedChunks?.map((e) => PoolChunkRange.fromPartial(e)) || [];
    message.requestFromOrigin = object.requestFromOrigin ?? false;
    return message;
  },
};

function createBasePoolMessage_RetractFileOfferData(): PoolMessage_RetractFileOfferData {
  return { fileId: "" };
}

export const PoolMessage_RetractFileOfferData = {
  encode(message: PoolMessage_RetractFileOfferData, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.fileId !== "") {
      writer.uint32(10).string(message.fileId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessage_RetractFileOfferData {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessage_RetractFileOfferData();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.fileId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessage_RetractFileOfferData {
    return { fileId: isSet(object.fileId) ? String(object.fileId) : "" };
  },

  toJSON(message: PoolMessage_RetractFileOfferData): unknown {
    const obj: any = {};
    message.fileId !== undefined && (obj.fileId = message.fileId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessage_RetractFileOfferData>, I>>(
    object: I,
  ): PoolMessage_RetractFileOfferData {
    const message = createBasePoolMessage_RetractFileOfferData();
    message.fileId = object.fileId ?? "";
    return message;
  },
};

function createBasePoolMessage_RetractFileRequestData(): PoolMessage_RetractFileRequestData {
  return { fileId: "" };
}

export const PoolMessage_RetractFileRequestData = {
  encode(message: PoolMessage_RetractFileRequestData, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.fileId !== "") {
      writer.uint32(10).string(message.fileId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessage_RetractFileRequestData {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessage_RetractFileRequestData();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.fileId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessage_RetractFileRequestData {
    return { fileId: isSet(object.fileId) ? String(object.fileId) : "" };
  },

  toJSON(message: PoolMessage_RetractFileRequestData): unknown {
    const obj: any = {};
    message.fileId !== undefined && (obj.fileId = message.fileId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessage_RetractFileRequestData>, I>>(
    object: I,
  ): PoolMessage_RetractFileRequestData {
    const message = createBasePoolMessage_RetractFileRequestData();
    message.fileId = object.fileId ?? "";
    return message;
  },
};

function createBasePoolDirectMessage(): PoolDirectMessage {
  return { type: 0, data: undefined };
}

export const PoolDirectMessage = {
  encode(message: PoolDirectMessage, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.type !== 0) {
      writer.uint32(8).int32(message.type);
    }
    if (message.data?.$case === "latestReplyData") {
      PoolDirectMessage_LatestReplyData.encode(message.data.latestReplyData, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolDirectMessage {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolDirectMessage();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.type = reader.int32() as any;
          break;
        case 2:
          message.data = {
            $case: "latestReplyData",
            latestReplyData: PoolDirectMessage_LatestReplyData.decode(reader, reader.uint32()),
          };
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolDirectMessage {
    return {
      type: isSet(object.type) ? poolDirectMessage_DirectTypeFromJSON(object.type) : 0,
      data: isSet(object.latestReplyData)
        ? {
          $case: "latestReplyData",
          latestReplyData: PoolDirectMessage_LatestReplyData.fromJSON(object.latestReplyData),
        }
        : undefined,
    };
  },

  toJSON(message: PoolDirectMessage): unknown {
    const obj: any = {};
    message.type !== undefined && (obj.type = poolDirectMessage_DirectTypeToJSON(message.type));
    message.data?.$case === "latestReplyData" && (obj.latestReplyData = message.data?.latestReplyData
      ? PoolDirectMessage_LatestReplyData.toJSON(message.data?.latestReplyData)
      : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolDirectMessage>, I>>(object: I): PoolDirectMessage {
    const message = createBasePoolDirectMessage();
    message.type = object.type ?? 0;
    if (
      object.data?.$case === "latestReplyData" &&
      object.data?.latestReplyData !== undefined &&
      object.data?.latestReplyData !== null
    ) {
      message.data = {
        $case: "latestReplyData",
        latestReplyData: PoolDirectMessage_LatestReplyData.fromPartial(object.data.latestReplyData),
      };
    }
    return message;
  },
};

function createBasePoolDirectMessage_LatestReplyData(): PoolDirectMessage_LatestReplyData {
  return { latestMessages: [], fileSeeders: [] };
}

export const PoolDirectMessage_LatestReplyData = {
  encode(message: PoolDirectMessage_LatestReplyData, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.latestMessages) {
      PoolMessage.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.fileSeeders) {
      PoolFileSeeders.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolDirectMessage_LatestReplyData {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolDirectMessage_LatestReplyData();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.latestMessages.push(PoolMessage.decode(reader, reader.uint32()));
          break;
        case 2:
          message.fileSeeders.push(PoolFileSeeders.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolDirectMessage_LatestReplyData {
    return {
      latestMessages: Array.isArray(object?.latestMessages)
        ? object.latestMessages.map((e: any) => PoolMessage.fromJSON(e))
        : [],
      fileSeeders: Array.isArray(object?.fileSeeders)
        ? object.fileSeeders.map((e: any) => PoolFileSeeders.fromJSON(e))
        : [],
    };
  },

  toJSON(message: PoolDirectMessage_LatestReplyData): unknown {
    const obj: any = {};
    if (message.latestMessages) {
      obj.latestMessages = message.latestMessages.map((e) => e ? PoolMessage.toJSON(e) : undefined);
    } else {
      obj.latestMessages = [];
    }
    if (message.fileSeeders) {
      obj.fileSeeders = message.fileSeeders.map((e) => e ? PoolFileSeeders.toJSON(e) : undefined);
    } else {
      obj.fileSeeders = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolDirectMessage_LatestReplyData>, I>>(
    object: I,
  ): PoolDirectMessage_LatestReplyData {
    const message = createBasePoolDirectMessage_LatestReplyData();
    message.latestMessages = object.latestMessages?.map((e) => PoolMessage.fromPartial(e)) || [];
    message.fileSeeders = object.fileSeeders?.map((e) => PoolFileSeeders.fromPartial(e)) || [];
    return message;
  },
};

function createBasePoolMessagePackageSourceInfo(): PoolMessagePackageSourceInfo {
  return { nodeId: "", path: [] };
}

export const PoolMessagePackageSourceInfo = {
  encode(message: PoolMessagePackageSourceInfo, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.nodeId !== "") {
      writer.uint32(10).string(message.nodeId);
    }
    writer.uint32(18).fork();
    for (const v of message.path) {
      writer.uint32(v);
    }
    writer.ldelim();
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessagePackageSourceInfo {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessagePackageSourceInfo();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.nodeId = reader.string();
          break;
        case 2:
          if ((tag & 7) === 2) {
            const end2 = reader.uint32() + reader.pos;
            while (reader.pos < end2) {
              message.path.push(reader.uint32());
            }
          } else {
            message.path.push(reader.uint32());
          }
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessagePackageSourceInfo {
    return {
      nodeId: isSet(object.nodeId) ? String(object.nodeId) : "",
      path: Array.isArray(object?.path) ? object.path.map((e: any) => Number(e)) : [],
    };
  },

  toJSON(message: PoolMessagePackageSourceInfo): unknown {
    const obj: any = {};
    message.nodeId !== undefined && (obj.nodeId = message.nodeId);
    if (message.path) {
      obj.path = message.path.map((e) => Math.round(e));
    } else {
      obj.path = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessagePackageSourceInfo>, I>>(object: I): PoolMessagePackageSourceInfo {
    const message = createBasePoolMessagePackageSourceInfo();
    message.nodeId = object.nodeId ?? "";
    message.path = object.path?.map((e) => e) || [];
    return message;
  },
};

function createBasePoolMessagePackageDestinationInfo(): PoolMessagePackageDestinationInfo {
  return { nodeId: "" };
}

export const PoolMessagePackageDestinationInfo = {
  encode(message: PoolMessagePackageDestinationInfo, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.nodeId !== "") {
      writer.uint32(10).string(message.nodeId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessagePackageDestinationInfo {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessagePackageDestinationInfo();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.nodeId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessagePackageDestinationInfo {
    return { nodeId: isSet(object.nodeId) ? String(object.nodeId) : "" };
  },

  toJSON(message: PoolMessagePackageDestinationInfo): unknown {
    const obj: any = {};
    message.nodeId !== undefined && (obj.nodeId = message.nodeId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessagePackageDestinationInfo>, I>>(
    object: I,
  ): PoolMessagePackageDestinationInfo {
    const message = createBasePoolMessagePackageDestinationInfo();
    message.nodeId = object.nodeId ?? "";
    return message;
  },
};

function createBasePoolChunkMessage(): PoolChunkMessage {
  return { fileId: "", chunkNumber: 0, chunk: new Uint8Array() };
}

export const PoolChunkMessage = {
  encode(message: PoolChunkMessage, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.fileId !== "") {
      writer.uint32(10).string(message.fileId);
    }
    if (message.chunkNumber !== 0) {
      writer.uint32(16).uint64(message.chunkNumber);
    }
    if (message.chunk.length !== 0) {
      writer.uint32(26).bytes(message.chunk);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolChunkMessage {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolChunkMessage();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.fileId = reader.string();
          break;
        case 2:
          message.chunkNumber = longToNumber(reader.uint64() as Long);
          break;
        case 3:
          message.chunk = reader.bytes();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolChunkMessage {
    return {
      fileId: isSet(object.fileId) ? String(object.fileId) : "",
      chunkNumber: isSet(object.chunkNumber) ? Number(object.chunkNumber) : 0,
      chunk: isSet(object.chunk) ? bytesFromBase64(object.chunk) : new Uint8Array(),
    };
  },

  toJSON(message: PoolChunkMessage): unknown {
    const obj: any = {};
    message.fileId !== undefined && (obj.fileId = message.fileId);
    message.chunkNumber !== undefined && (obj.chunkNumber = Math.round(message.chunkNumber));
    message.chunk !== undefined &&
      (obj.chunk = base64FromBytes(message.chunk !== undefined ? message.chunk : new Uint8Array()));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolChunkMessage>, I>>(object: I): PoolChunkMessage {
    const message = createBasePoolChunkMessage();
    message.fileId = object.fileId ?? "";
    message.chunkNumber = object.chunkNumber ?? 0;
    message.chunk = object.chunk ?? new Uint8Array();
    return message;
  },
};

function createBasePoolMessagePackage(): PoolMessagePackage {
  return {
    src: undefined,
    dests: [],
    partnerIntPath: undefined,
    msg: undefined,
    directMsg: undefined,
    chunkMsg: undefined,
  };
}

export const PoolMessagePackage = {
  encode(message: PoolMessagePackage, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.src !== undefined) {
      PoolMessagePackageSourceInfo.encode(message.src, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.dests) {
      PoolMessagePackageDestinationInfo.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    if (message.partnerIntPath !== undefined) {
      writer.uint32(24).uint32(message.partnerIntPath);
    }
    if (message.msg !== undefined) {
      PoolMessage.encode(message.msg, writer.uint32(34).fork()).ldelim();
    }
    if (message.directMsg !== undefined) {
      PoolDirectMessage.encode(message.directMsg, writer.uint32(42).fork()).ldelim();
    }
    if (message.chunkMsg !== undefined) {
      PoolChunkMessage.encode(message.chunkMsg, writer.uint32(50).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PoolMessagePackage {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePoolMessagePackage();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.src = PoolMessagePackageSourceInfo.decode(reader, reader.uint32());
          break;
        case 2:
          message.dests.push(PoolMessagePackageDestinationInfo.decode(reader, reader.uint32()));
          break;
        case 3:
          message.partnerIntPath = reader.uint32();
          break;
        case 4:
          message.msg = PoolMessage.decode(reader, reader.uint32());
          break;
        case 5:
          message.directMsg = PoolDirectMessage.decode(reader, reader.uint32());
          break;
        case 6:
          message.chunkMsg = PoolChunkMessage.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): PoolMessagePackage {
    return {
      src: isSet(object.src) ? PoolMessagePackageSourceInfo.fromJSON(object.src) : undefined,
      dests: Array.isArray(object?.dests)
        ? object.dests.map((e: any) => PoolMessagePackageDestinationInfo.fromJSON(e))
        : [],
      partnerIntPath: isSet(object.partnerIntPath) ? Number(object.partnerIntPath) : undefined,
      msg: isSet(object.msg) ? PoolMessage.fromJSON(object.msg) : undefined,
      directMsg: isSet(object.directMsg) ? PoolDirectMessage.fromJSON(object.directMsg) : undefined,
      chunkMsg: isSet(object.chunkMsg) ? PoolChunkMessage.fromJSON(object.chunkMsg) : undefined,
    };
  },

  toJSON(message: PoolMessagePackage): unknown {
    const obj: any = {};
    message.src !== undefined && (obj.src = message.src ? PoolMessagePackageSourceInfo.toJSON(message.src) : undefined);
    if (message.dests) {
      obj.dests = message.dests.map((e) => e ? PoolMessagePackageDestinationInfo.toJSON(e) : undefined);
    } else {
      obj.dests = [];
    }
    message.partnerIntPath !== undefined && (obj.partnerIntPath = Math.round(message.partnerIntPath));
    message.msg !== undefined && (obj.msg = message.msg ? PoolMessage.toJSON(message.msg) : undefined);
    message.directMsg !== undefined &&
      (obj.directMsg = message.directMsg ? PoolDirectMessage.toJSON(message.directMsg) : undefined);
    message.chunkMsg !== undefined &&
      (obj.chunkMsg = message.chunkMsg ? PoolChunkMessage.toJSON(message.chunkMsg) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<PoolMessagePackage>, I>>(object: I): PoolMessagePackage {
    const message = createBasePoolMessagePackage();
    message.src = (object.src !== undefined && object.src !== null)
      ? PoolMessagePackageSourceInfo.fromPartial(object.src)
      : undefined;
    message.dests = object.dests?.map((e) => PoolMessagePackageDestinationInfo.fromPartial(e)) || [];
    message.partnerIntPath = object.partnerIntPath ?? undefined;
    message.msg = (object.msg !== undefined && object.msg !== null) ? PoolMessage.fromPartial(object.msg) : undefined;
    message.directMsg = (object.directMsg !== undefined && object.directMsg !== null)
      ? PoolDirectMessage.fromPartial(object.directMsg)
      : undefined;
    message.chunkMsg = (object.chunkMsg !== undefined && object.chunkMsg !== null)
      ? PoolChunkMessage.fromPartial(object.chunkMsg)
      : undefined;
    return message;
  },
};

declare var self: any | undefined;
declare var window: any | undefined;
declare var global: any | undefined;
var tsProtoGlobalThis: any = (() => {
  if (typeof globalThis !== "undefined") {
    return globalThis;
  }
  if (typeof self !== "undefined") {
    return self;
  }
  if (typeof window !== "undefined") {
    return window;
  }
  if (typeof global !== "undefined") {
    return global;
  }
  throw "Unable to locate global object";
})();

function bytesFromBase64(b64: string): Uint8Array {
  if (tsProtoGlobalThis.Buffer) {
    return Uint8Array.from(tsProtoGlobalThis.Buffer.from(b64, "base64"));
  } else {
    const bin = tsProtoGlobalThis.atob(b64);
    const arr = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; ++i) {
      arr[i] = bin.charCodeAt(i);
    }
    return arr;
  }
}

function base64FromBytes(arr: Uint8Array): string {
  if (tsProtoGlobalThis.Buffer) {
    return tsProtoGlobalThis.Buffer.from(arr).toString("base64");
  } else {
    const bin: string[] = [];
    arr.forEach((byte) => {
      bin.push(String.fromCharCode(byte));
    });
    return tsProtoGlobalThis.btoa(bin.join(""));
  }
}

type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;

export type DeepPartial<T> = T extends Builtin ? T
  : T extends Array<infer U> ? Array<DeepPartial<U>> : T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>>
  : T extends { $case: string } ? { [K in keyof Omit<T, "$case">]?: DeepPartial<T[K]> } & { $case: T["$case"] }
  : T extends {} ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type Exact<P, I extends P> = P extends Builtin ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & { [K in Exclude<keyof I, KeysOfUnion<P>>]: never };

function longToNumber(long: Long): number {
  if (long.gt(Number.MAX_SAFE_INTEGER)) {
    throw new tsProtoGlobalThis.Error("Value is larger than Number.MAX_SAFE_INTEGER");
  }
  return long.toNumber();
}

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}

function isSet(value: any): boolean {
  return value !== null && value !== undefined;
}
