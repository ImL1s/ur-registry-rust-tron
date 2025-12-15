import 'package:ur_registry_flutter/ffi/ffi_factory.dart';
import 'package:ur_registry_flutter/native_object.dart';
import 'package:ur_registry_flutter/response.dart';
import 'package:ur_registry_flutter/ur_encoder.dart';
import 'package:uuid/uuid.dart';
import 'package:convert/convert.dart';

const nativePrefix = "tron_sign_request";

typedef NativeConstruct = Pointer<Response> Function(Pointer<Utf8>,
    Pointer<Utf8>, Pointer<Utf8>, Uint32, Pointer<Utf8>, Pointer<Utf8>, Uint32);
typedef Construct = Pointer<Response> Function(Pointer<Utf8>, Pointer<Utf8>,
    Pointer<Utf8>, int, Pointer<Utf8>, Pointer<Utf8>, int);
typedef NativeGetUREncoder = Pointer<Response> Function(Pointer<Void>);
typedef NativeGetRequestId = Pointer<Response> Function(Pointer<Void>);
typedef NativeGetSignData = Pointer<Response> Function(Pointer<Void>);
typedef NativeGetDerivationPath = Pointer<Response> Function(Pointer<Void>);
typedef NativeNew = Pointer<Response> Function();

/// TRON 簽名請求
///
/// 用於生成 Keystone 硬件錢包可識別的 UR 編碼 QR 碼
///
/// 使用方式:
/// ```dart
/// final request = TronSignRequest.factory(
///   signData: unsignedTxBytes,
///   path: "m/44'/195'/0'/0/0",
///   xfp: "12345678",
///   address: "TRxxx...",
///   origin: "TRON MultiSig Wallet",
///   dataType: TronSignRequest.transaction,
/// );
/// final urEncoder = request.toUREncoder();
/// // 使用 urEncoder 生成 QR 碼
/// ```
class TronSignRequest extends NativeObject {
  /// 交易類型
  static int transaction = 1;

  /// 消息類型
  static int message = 2;

  /// TypedData 類型
  static int typedData = 3;

  late Construct nativeConstruct = lib
      .lookup<NativeFunction<NativeConstruct>>("${nativePrefix}_construct")
      .asFunction<Construct>();
  late NativeGetUREncoder nativeGetUREncoder = lib
      .lookup<NativeFunction<NativeGetUREncoder>>(
          "${nativePrefix}_get_ur_encoder")
      .asFunction();
  late NativeNew nativeNew =
      lib.lookup<NativeFunction<NativeNew>>("${nativePrefix}_new").asFunction();
  late NativeGetRequestId nativeGetRequestId = lib
      .lookup<NativeFunction<NativeGetRequestId>>(
          "${nativePrefix}_get_request_id")
      .asFunction();
  late NativeGetSignData nativeGetSignData = lib
      .lookup<NativeFunction<NativeGetSignData>>(
          "${nativePrefix}_get_sign_data")
      .asFunction();
  late NativeGetDerivationPath nativeGetDerivationPath = lib
      .lookup<NativeFunction<NativeGetDerivationPath>>(
          "${nativePrefix}_get_derivation_path")
      .asFunction();

  late String uuid;

  /// 從原生對象創建
  TronSignRequest(Pointer<Void> object) : super() {
    nativeObject = object;
    final response = nativeGetRequestId(nativeObject).ref;
    final uuidBuffer = response.getString();
    uuid = Uuid.unparse(hex.decode(uuidBuffer));
  }

  /// 創建新的 TRON 簽名請求
  ///
  /// [signData] 未簽名交易或消息的字節數組
  /// [path] BIP44 派生路徑，例如 "m/44'/195'/0'/0/0"
  /// [xfp] 主指紋（16進制字符串）
  /// [address] TRON 地址（可選）
  /// [origin] 請求來源標識（可選）
  /// [dataType] 數據類型：1=交易, 2=消息, 3=TypedData
  TronSignRequest.factory({
    required List<int> signData,
    required String path,
    required String xfp,
    String? address,
    String origin = 'TRON MultiSig Wallet',
    int dataType = 1,
  }) : super() {
    uuid = const Uuid().v4();
    final buffer = Uuid.parse(uuid);
    final uuidBufferStr = hex.encode(buffer);
    final signDataStr = hex.encode(signData);
    final xfpInt = int.parse(xfp, radix: 16);

    final response = nativeConstruct(
            uuidBufferStr.toNativeUtf8(),
            signDataStr.toNativeUtf8(),
            path.toNativeUtf8(),
            xfpInt,
            (address ?? '').toNativeUtf8(),
            origin.toNativeUtf8(),
            dataType)
        .ref;
    nativeObject = response.getObject();
  }

  /// 獲取 UR 編碼器
  ///
  /// 返回的 UREncoder 可用於生成動態 QR 碼
  UREncoder toUREncoder() {
    final response = nativeGetUREncoder(nativeObject).ref;
    return UREncoder(response.getObject());
  }

  /// 獲取請求 ID
  String getRequestId() {
    return uuid;
  }

  /// 獲取簽名數據（16進制）
  String getSignData() {
    final response = nativeGetSignData(nativeObject).ref;
    return response.getString();
  }

  /// 獲取派生路徑
  String getDerivationPath() {
    final response = nativeGetDerivationPath(nativeObject).ref;
    return response.getString();
  }
}
