import base64
import hashlib
import os

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec


def required(name: str) -> str:
    value = os.environ.get(name)
    if not value:
        raise RuntimeError(f"{name} is required")
    return value


delta_bytes = base64.b64decode(required("SYNC_VERIFY_DELTA_BASE64"), validate=True)
expected_hash = required("SYNC_VERIFY_CONTENT_HASH")
actual_hash = hashlib.sha256(delta_bytes).hexdigest()
if actual_hash != expected_hash:
    raise RuntimeError("downloaded delta hash does not match the manifest")

public_key = serialization.load_pem_public_key(
    required("SYNC_SIGNING_PUBLIC_KEY").encode("utf-8")
)
if not isinstance(public_key, ec.EllipticCurvePublicKey):
    raise RuntimeError("sync public key is not an EC public key")
if public_key.curve.name != "secp256r1":
    raise RuntimeError("sync public key is not P-256")

try:
    public_key.verify(
        base64.b64decode(required("SYNC_VERIFY_SIGNATURE"), validate=True),
        expected_hash.encode("ascii"),
        ec.ECDSA(hashes.SHA256()),
    )
except InvalidSignature as error:
    raise RuntimeError("delta signature verification failed") from error

print("delta_hash=verified")
print("delta_signature=verified_p256_sha256")
