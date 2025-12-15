import 'package:ur_registry_flutter/ffi/ffi_factory.dart';
import 'package:ur_registry_flutter/native_object.dart';
import 'package:ur_registry_flutter/response.dart';
import 'package:uuid/uuid.dart';
import 'package:convert/convert.dart';

const nativePrefix = "tron_signature";

typedef NativeGetSignature = Pointer<Response> Function(Pointer<Void>);
typedef NativeGetRequestId = Pointer<Response> Function(Pointer<Void>);

/// TRON 簽名結果
///
/// 從 Keystone 硬件錢包掃描的簽名 QR 碼中解析
///
/// 使用方式:
/// ```dart
/// // 在 AnimatedQRScanner 的 onSuccess 回調中
/// void onScanSuccess(NativeObject object) {
///   final tronSignature = object as TronSignature;
///   final signature = tronSignature.getSignature();
///   final requestId = tronSignature.getRequestId();
///   // 使用簽名完成交易
/// }
/// ```
class TronSignature extends NativeObject {
  late NativeGetSignature nativeGetSignature = lib
      .lookup<NativeFunction<NativeGetSignature>>(
          "${nativePrefix}_get_signature")
      .asFunction();
  late NativeGetRequestId nativeGetRequestId = lib
      .lookup<NativeFunction<NativeGetRequestId>>(
          "${nativePrefix}_get_request_id")
      .asFunction();

  /// 從原生對象創建
  TronSignature(Pointer<Void> object) : super() {
    nativeObject = object;
  }

  /// 獲取簽名數據（16進制字符串）
  ///
  /// 返回 65 字節的 ECDSA 簽名：r(32) + s(32) + v(1)
  String getSignature() {
    final response = nativeGetSignature(nativeObject).ref;
    return response.getString();
  }

  /// 獲取簽名對應的請求 ID（16進制字符串）
  ///
  /// 用於匹配簽名請求和簽名結果
  String getRequestId() {
    final response = nativeGetRequestId(nativeObject).ref;
    return response.getString();
  }

  /// 獲取簽名對應的請求 UUID
  String getRequestUUID() {
    final requestIdHex = getRequestId();
    return Uuid.unparse(hex.decode(requestIdHex));
  }
}
