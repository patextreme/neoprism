POST http://localhost:8090/v2/wallets
{
  "name": "Test wallet",
  "mnemonic_sentence": [
    "mimic",
    "candy",
    "diamond",
    "virus",
    "hospital",
    "dragon",
    "culture",
    "price",
    "emotion",
    "tell",
    "update",
    "give",
    "faint",
    "resist",
    "faculty",
    "soup",
    "demand",
    "window",
    "dignity",
    "capital",
    "bullet",
    "purity",
    "practice",
    "fossil"
  ],
  "passphrase": "super_secret"
}
HTTP 201
[Captures]
wallet_id: jsonpath "$.id"

GET http://localhost:8090/v2/wallets/{{ wallet_id }}/addresses
[Options]
verbose: true
HTTP 200
[Captures]
address: jsonpath "$[0].id"
