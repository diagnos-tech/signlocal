/**
 * The fake's key pairs. PUBLIC TEST KEYS: they are published in this package,
 * so a signature made with them proves nothing, which is the point. Fixed
 * rather than generated so certificates, fingerprints and PKCS#1 signatures
 * are identical on every run (snapshot-friendly). Generated once with
 * Node's crypto.generateKeyPairSync; never reuse them outside tests.
 */

/** An EC P-256 private key (JWK). */
export const TEST_EC_KEY: JsonWebKey = {
  kty: "EC",
  crv: "P-256",
  x: "6TicrXR_37yFFKLLBw15MYOikplQuvCrmf9tOSECQfM",
  y: "i-cd7eZgIlKHkmqnkdsyy9vVGNQ15fWWt-whcRxw-VQ",
  d: "toLkbvU2EJVS4QF3KKFoMZyWWGWNX7FcxUCxtpOO1WQ",
};

/** An RSA-2048 private key (JWK), with its CRT parameters. */
export const TEST_RSA_KEY: JsonWebKey = {
  kty: "RSA",
  n: "v-e4lCntng5EiAod_jicekPNLPFWBg6m0FxFTqW9W579HXzZLTWnRs5JKw2xQSxsQFy3KpSf6lbATahjF2TKBRBAffuwrDISt0lRbE7FFsvmj9kGp6bpqZnTmvilEWwFUSbUnkYb-GzDboFcpBut2_gxEU6JP-imofjMOSwQ_Rsbrr66gPGgprE9pUlVFwGYQ5MEfVPWgVZicJrrEMgqJDukrSpdjD-ebyTthh3FSuhzkdI_QmFu431lE-UJOrkv14JsL0U6G0vuB3rYC1-thVihGqUCYU1yskc2fx7ETKoXBeUbUUX-nYApU0QZWA77JRC_OHgXtQJIYzbovBohBQ",
  e: "AQAB",
  d: "RQZfXSeRmMKwArCE5j5Nhiqh_3LUyrv1Y0d1BFtX9z8B8tvHr9u3DaBAMBSuN293hlTy6wVnWZ4XcDdLYQw3H8gfMlFX1C_0jGl6OHdCq7bueLoKiz3dmMvEEV8y3Efax7wsSLuV7u6MAtDT1hFAoTityXhSpKKVYPgA2OWRPMAmZjf0qcgds-1LtKmvBUFjrY2We_gB40hc9OO8WnHLuOytfe8fFT1T34tj0ej4rJ0PVvg8YXV55-7kvJXUr-ihr2T-oNRrcFM7LXHpOcVTEqr7nUSu3DdQo8eqbcCyE9Rby1ocs1YJo7PK2sE-Y6K6UV8Sg43dr0p6JUxf_TIoNw",
  p: "4U3rj1kHrW4US2s2GkXXsj8JzwfkiYRx-6oyiyzKvkMvJiSBDecxiwoIattVLwvi9ky2DJRTbUWOTzShJqmqR7D25omRABtVXmY8g1OJZLH9Qdml9ZPZG0qWkphuoJBeuv-lNHV7TER-zKPerH4qSuBQEdYb0fiva_gdUEQzNR8",
  q: "2gzqcW3a-sYda33HO--xQnnVSfep9S-mTmFmsa0XJxKM32CvHv-V-l0huuy3sW2ZEWKh7lcs5Nl3P2gn-h07TrED0bwx3UHcRizybuT5_-kwyVlaL8oc8jyT1N7JNOIkfsZErf7eC1JswdBh-b2eJEqfrLWjGBKVc1JygncK4Vs",
  dp: "bUuPagqWXtt3nN51cGlRKXbLG1OI2e0WHi-CNWGVOJb-2FH9CPGOZxSG4M9lk10AzNqJtbB-iBOl5WZyhn7ny79dBpjVbmfUEecmk86CaAawBLHAgFEokMSuJo8rm1htm-lICxqypVzU5hDYhHTvr0MBlKV4-XLaJlZGYhmzfaE",
  dq: "VtTh332xwD94o7-YwWN4IVLG_wejJMJu5bOf441cNPEMLEAiPKPnI6ZGsltl40ATvnqapYENnayD8joWAWH2piWTscfRy5xYjZGEkWiZrbE8_lGYuoMv977r189ILMkJY3qtCuT9o2CVcrD68yrxeN5Gq85BPmrYHO5uL0ociRU",
  qi: "E-HrwREFDmwhq8N18nTmRb92WiLPU4bg35Y8LFWQj1lvqUUtVTjSqfz9zPJi-nIdlfgi38AMsbaMS-CJQGZ7JlwDe2WuFZgSFm-eN0ZBaT5w89fKfyMgpgBIHDESt0xK0Pp_N4DcOQSmIEBh5zlMdMfR_ZVhgDm6YwuFwbgJGG0",
};
