Feature: Offline
  Scenario: Encode and decode addresses
    When I generate a key
    And I decode the address
    And I encode the address
    Then the address should still be the same

  Scenario Outline: Mnemonic to and from private key
    Given mnemonic for private key "<mn>"
    When I convert the private key back to a mnemonic
    Then the mnemonic should still be the same as "<mn>"

    Examples:
    | mn                                                                                                                                                                   |
    | advice pudding treat near rule blouse same whisper inner electric quit surface sunny dismiss leader blood seat clown cost exist hospital century reform able sponsor |

  Scenario Outline: Mnemonic to and from master derivation key
    Given mnemonic for master derivation key "<mn>"
    When I convert the master derivation key back to a mnemonic
    Then the mnemonic should still be the same as "<mn>"

    Examples:
    | mn                                                                                                                                                                   |
    | advice pudding treat near rule blouse same whisper inner electric quit surface sunny dismiss leader blood seat clown cost exist hospital century reform able sponsor |

  Scenario Outline: Sign transaction
    Given payment transaction parameters <fee> <fv> <lv> "<gh>" "<to>" "<close>" <amt> "<gen>" "<note>"
    And mnemonic for private key "<mn>"
    When I create the payment transaction
    And I sign the transaction with the private key
    Then the signed transaction should equal the golden "<golden>"

    Examples:
    | fee | fv    | lv    | gh                                           | to                                                         | close                                                      | amt  | gen          | note         | mn                                                                                                                                                                   | golden                                                                                                                                                                                                                                                                                                                                                                                                       |
    | 4   | 12466 | 13466 | JgsgCaCTqIaLeVhyL6XlRu3n7Rfk2FxMeK+wRSaQ7dI= | PNWOET7LLOWMBMLE4KOCELCX6X3D3Q4H2Q4QJASYIEOF7YIPPQBG3YQ5YI | IDUTJEUIEVSMXTU4LGTJWZ2UE2E6TIODUKU6UW3FU3UKIQQ77RLUBBBFLA | 1000 | devnet-v33.0 | 6gAVR0Nsv5Y= | advice pudding treat near rule blouse same whisper inner electric quit surface sunny dismiss leader blood seat clown cost exist hospital century reform able sponsor | gqNzaWfEQPhUAZ3xkDDcc8FvOVo6UinzmKBCqs0woYSfodlmBMfQvGbeUx3Srxy3dyJDzv7rLm26BRv9FnL2/AuT7NYfiAWjdHhui6NhbXTNA+ilY2xvc2XEIEDpNJKIJWTLzpxZpptnVCaJ6aHDoqnqW2Wm6KRCH/xXo2ZlZc0EmKJmds0wsqNnZW6sZGV2bmV0LXYzMy4womdoxCAmCyAJoJOohot5WHIvpeVG7eftF+TYXEx4r7BFJpDt0qJsds00mqRub3RlxAjqABVHQ2y/lqNyY3bEIHts4k/rW6zAsWTinCIsV/X2PcOH1DkEglhBHF/hD3wCo3NuZMQg5/D4TQaBHfnzHI2HixFV9GcdUaGFwgCQhmf0SVhwaKGkdHlwZaNwYXk= |

  Scenario Outline: Sign transaction with flat fee
    Given payment transaction parameters <fee> <fv> <lv> "<gh>" "<to>" "<close>" <amt> "<gen>" "<note>"
    And mnemonic for private key "<mn>"
    When I create the flat fee payment transaction
    And I sign the transaction with the private key
    Then the signed transaction should equal the golden "<golden>"

    Examples:
    | fee  | fv    | lv    | gh                                           | to                                                         | close                                                      | amt  | gen          | note         | mn                                                                                                                                                                   | golden                                                                                                                                                                                                                                                                                                                                                                                                       |
    | 1176 | 12466 | 13466 | JgsgCaCTqIaLeVhyL6XlRu3n7Rfk2FxMeK+wRSaQ7dI= | PNWOET7LLOWMBMLE4KOCELCX6X3D3Q4H2Q4QJASYIEOF7YIPPQBG3YQ5YI | IDUTJEUIEVSMXTU4LGTJWZ2UE2E6TIODUKU6UW3FU3UKIQQ77RLUBBBFLA | 1000 | devnet-v33.0 | 6gAVR0Nsv5Y= | advice pudding treat near rule blouse same whisper inner electric quit surface sunny dismiss leader blood seat clown cost exist hospital century reform able sponsor | gqNzaWfEQPhUAZ3xkDDcc8FvOVo6UinzmKBCqs0woYSfodlmBMfQvGbeUx3Srxy3dyJDzv7rLm26BRv9FnL2/AuT7NYfiAWjdHhui6NhbXTNA+ilY2xvc2XEIEDpNJKIJWTLzpxZpptnVCaJ6aHDoqnqW2Wm6KRCH/xXo2ZlZc0EmKJmds0wsqNnZW6sZGV2bmV0LXYzMy4womdoxCAmCyAJoJOohot5WHIvpeVG7eftF+TYXEx4r7BFJpDt0qJsds00mqRub3RlxAjqABVHQ2y/lqNyY3bEIHts4k/rW6zAsWTinCIsV/X2PcOH1DkEglhBHF/hD3wCo3NuZMQg5/D4TQaBHfnzHI2HixFV9GcdUaGFwgCQhmf0SVhwaKGkdHlwZaNwYXk= |

  Scenario Outline: Multisig address
    Given multisig addresses "<addresses>"
    Then the multisig address should equal the golden "<golden>"

    Examples:
    | addresses                                                                                                                                                                        | golden                                                     |
    | DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA BFRTECKTOOE7A5LHCF3TTEOH2A7BW46IYT2SX5VP6ANKEXHZYJY77SJTVM 47YPQTIGQEO7T4Y4RWDYWEKV6RTR2UNBQXBABEEGM72ESWDQNCQ52OPASU | RWJLJCMQAFZ2ATP2INM2GZTKNL6OULCCUBO5TQPXH3V2KR4AG7U5UA5JNM |

  Scenario Outline: Sign multisig
    Given payment transaction parameters <fee> <fv> <lv> "<gh>" "<to>" "<close>" <amt> "<gen>" "<note>"
    And mnemonic for private key "<mn>"
    And multisig addresses "<addresses>"
    When I create the multisig payment transaction
    And I sign the multisig transaction with the private key
    Then the multisig transaction should equal the golden "<golden>"

    Examples:
    | fee | fv    | lv    | gh                                           | to                                                         | close                                                      | amt  | gen          | note         | mn                                                                                                                                                                   | addresses                                                                                                                                                                        | golden                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
    | 4   | 12466 | 13466 | JgsgCaCTqIaLeVhyL6XlRu3n7Rfk2FxMeK+wRSaQ7dI= | PNWOET7LLOWMBMLE4KOCELCX6X3D3Q4H2Q4QJASYIEOF7YIPPQBG3YQ5YI | IDUTJEUIEVSMXTU4LGTJWZ2UE2E6TIODUKU6UW3FU3UKIQQ77RLUBBBFLA | 1000 | devnet-v33.0 | X4Bl4wQ9rCo= | advice pudding treat near rule blouse same whisper inner electric quit surface sunny dismiss leader blood seat clown cost exist hospital century reform able sponsor | DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA BFRTECKTOOE7A5LHCF3TTEOH2A7BW46IYT2SX5VP6ANKEXHZYJY77SJTVM 47YPQTIGQEO7T4Y4RWDYWEKV6RTR2UNBQXBABEEGM72ESWDQNCQ52OPASU | gqRtc2lng6ZzdWJzaWeTgaJwa8QgG37AsEvqYbeWkJfmy/QH4QinBTUdC8mKvrEiCairgXiBonBrxCAJYzIJU3OJ8HVnEXc5kcfQPhtzyMT1K/av8BqiXPnCcYKicGvEIOfw+E0GgR358xyNh4sRVfRnHVGhhcIAkIZn9ElYcGihoXPEQF6nXZ7CgInd1h7NVspIPFZNhkPL+vGFpTNwH3Eh9gwPM8pf1EPTHfPvjf14sS7xN7mTK+wrz7Odhp4rdWBNUASjdGhyAqF2AaN0eG6Lo2FtdM0D6KVjbG9zZcQgQOk0koglZMvOnFmmm2dUJonpocOiqepbZabopEIf/FejZmVlzQSYomZ2zTCyo2dlbqxkZXZuZXQtdjMzLjCiZ2jEICYLIAmgk6iGi3lYci+l5Ubt5+0X5NhcTHivsEUmkO3Somx2zTSapG5vdGXECF+AZeMEPawqo3JjdsQge2ziT+tbrMCxZOKcIixX9fY9w4fUOQSCWEEcX+EPfAKjc25kxCCNkrSJkAFzoE36Q1mjZmpq/OosQqBd2cH3PuulR4A36aR0eXBlo3BheQ== |

  Scenario Outline: Append multisig
    Given encoded multisig transaction "<mtx>"
    And mnemonic for private key "<mn>"
    When I append a signature to the multisig transaction
    Then the multisig transaction should equal the golden "<golden>"

    Examples:
    | mtx                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | mn                                                                                                                                                                | golden                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
    | gqRtc2lng6ZzdWJzaWeTgqJwa8QgG37AsEvqYbeWkJfmy/QH4QinBTUdC8mKvrEiCairgXihc8RAuLAFE0oma0skOoAmOzEwfPuLYpEWl4LINtsiLrUqWQkDxh4WHb29//YCpj4MFbiSgD2jKYt0XKRD86zKCF4RDYGicGvEIAljMglTc4nwdWcRdzmRx9A+G3PIxPUr9q/wGqJc+cJxgaJwa8Qg5/D4TQaBHfnzHI2HixFV9GcdUaGFwgCQhmf0SVhwaKGjdGhyAqF2AaN0eG6Lo2FtdM0D6KVjbG9zZcQgQOk0koglZMvOnFmmm2dUJonpocOiqepbZabopEIf/FejZmVlzQPoomZ2zfMVo2dlbqxkZXZuZXQtdjM4LjCiZ2jEIP6zbDkQFDkAw9pVQsoYNrAP0vgZWRJXzSP2BC+YyDadomx2zfb9pG5vdGXECEUmIgAYUob7o3JjdsQge2ziT+tbrMCxZOKcIixX9fY9w4fUOQSCWEEcX+EPfAKjc25kxCCNkrSJkAFzoE36Q1mjZmpq/OosQqBd2cH3PuulR4A36aR0eXBlo3BheQ== | since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left | gqRtc2lng6ZzdWJzaWeTgqJwa8QgG37AsEvqYbeWkJfmy/QH4QinBTUdC8mKvrEiCairgXihc8RAuLAFE0oma0skOoAmOzEwfPuLYpEWl4LINtsiLrUqWQkDxh4WHb29//YCpj4MFbiSgD2jKYt0XKRD86zKCF4RDYKicGvEIAljMglTc4nwdWcRdzmRx9A+G3PIxPUr9q/wGqJc+cJxoXPEQBAhuyRjsOrnHp3s/xI+iMKiL7QPsh8iJZ22YOJJP0aFUwedMr+a6wfdBXk1OefyrAN1wqJ9rq6O+DrWV1fH0ASBonBrxCDn8PhNBoEd+fMcjYeLEVX0Zx1RoYXCAJCGZ/RJWHBooaN0aHICoXYBo3R4boujYW10zQPopWNsb3NlxCBA6TSSiCVky86cWaabZ1Qmiemhw6Kp6ltlpuikQh/8V6NmZWXNA+iiZnbN8xWjZ2VurGRldm5ldC12MzguMKJnaMQg/rNsORAUOQDD2lVCyhg2sA/S+BlZElfNI/YEL5jINp2ibHbN9v2kbm90ZcQIRSYiABhShvujcmN2xCB7bOJP61uswLFk4pwiLFf19j3Dh9Q5BIJYQRxf4Q98AqNzbmTEII2StImQAXOgTfpDWaNmamr86ixCoF3Zwfc+66VHgDfppHR5cGWjcGF5 |

  Scenario Outline: Merge multisig
    Given encoded multisig transactions "<msigtxns>"
    When I merge the multisig transactions
    Then the multisig transaction should equal the golden "<golden>"

    Examples:
    | msigtxns | golden |
    | gqRtc2lng6ZzdWJzaWeTgqJwa8QgphunEajorK/Yj00fDOcOo1TXKQMvhe6frJxwipP1yiKhc8RA+f+fqZgjzOKV1Y8RlHxk0R5InGx5jsnF1gbKXVq+pAxwqSvtSTjTM7mRY0zH7tbv0dJtcuturoLbmX3lRWZCD4GicGvEIM9tutXmHvqZsk/Hk65YFjn348EccLQrgf9Kp0bzsvnUgaJwa8QgegKRmOgvSz67ItDrNQquyDe17UTgWictMvtqYfpYGCijdGhyAqF2AaN0eG6Io2FtdM0D6KNmZWXNA+iiZnYBomdoxCD+s2w5EBQ5AMPaVULKGDawD9L4GVkSV80j9gQvmMg2naJsds0D6KNyY3bEII4yNZs+IAqmxwEyX1cl45jSec8y0gubN5/lTYQPr95eo3NuZMQgkC7TLOEydGApKJoTita0Z+7jHVqj74oYHwVgXX1YjSKkdHlwZaNwYXk= gqRtc2lng6ZzdWJzaWeTgaJwa8QgphunEajorK/Yj00fDOcOo1TXKQMvhe6frJxwipP1yiKConBrxCDPbbrV5h76mbJPx5OuWBY59+PBHHC0K4H/SqdG87L51KFzxEBfG9erywuPXY/DsgOsadIqou7676GhGH4oSX5K2iSLDCf8L0pFoS3Hmepjsy8FcY62AFIL3Vg5lQLxTdlF670NgaJwa8QgegKRmOgvSz67ItDrNQquyDe17UTgWictMvtqYfpYGCijdGhyAqF2AaN0eG6Io2FtdM0D6KNmZWXNA+iiZnYBomdoxCD+s2w5EBQ5AMPaVULKGDawD9L4GVkSV80j9gQvmMg2naJsds0D6KNyY3bEII4yNZs+IAqmxwEyX1cl45jSec8y0gubN5/lTYQPr95eo3NuZMQgkC7TLOEydGApKJoTita0Z+7jHVqj74oYHwVgXX1YjSKkdHlwZaNwYXk= | gqRtc2lng6ZzdWJzaWeTgqJwa8QgphunEajorK/Yj00fDOcOo1TXKQMvhe6frJxwipP1yiKhc8RA+f+fqZgjzOKV1Y8RlHxk0R5InGx5jsnF1gbKXVq+pAxwqSvtSTjTM7mRY0zH7tbv0dJtcuturoLbmX3lRWZCD4KicGvEIM9tutXmHvqZsk/Hk65YFjn348EccLQrgf9Kp0bzsvnUoXPEQF8b16vLC49dj8OyA6xp0iqi7vrvoaEYfihJfkraJIsMJ/wvSkWhLceZ6mOzLwVxjrYAUgvdWDmVAvFN2UXrvQ2BonBrxCB6ApGY6C9LPrsi0Os1Cq7IN7XtROBaJy0y+2ph+lgYKKN0aHICoXYBo3R4boijYW10zQPoo2ZlZc0D6KJmdgGiZ2jEIP6zbDkQFDkAw9pVQsoYNrAP0vgZWRJXzSP2BC+YyDadomx2zQPoo3JjdsQgjjI1mz4gCqbHATJfVyXjmNJ5zzLSC5s3n+VNhA+v3l6jc25kxCCQLtMs4TJ0YCkomhOK1rRn7uMdWqPvihgfBWBdfViNIqR0eXBlo3BheQ== |

  Scenario Outline: Microalgos to algos
    When I convert <microalgos> microalgos to algos and back
    Then it should still be the same amount of microalgos <microalgos>

    Examples:
    | microalgos   |
    | 123456789012 |
    | 123456789013 |
    | 123456789014 |
    | 123456789015 |
    | 123456789016 |
    | 123456789017 |

  Scenario Outline: Create key registration transaction
    Given key registration transaction parameters <fee> <fv> <lv> "<gh>" "<votekey>" "<selkey>" <votefst> <votelst> <votekd> "<gen>" "<note>"
    And mnemonic for private key "<mn>"
    When I create the key registration transaction
    And I sign the transaction with the private key
    Then the signed transaction should equal the golden "<golden>"

    Examples:
    | fee | fv    | lv    | gh                                           | votekey                                      | selkey                                       | votefst | votelst | votekd | gen          | note         | mn                                                                                                                                                                   | golden                                                                                                                                                                                                                                                                                                                                                                                                                                               |
    | 4   | 12466 | 13466 | JgsgCaCTqIaLeVhyL6XlRu3n7Rfk2FxMeK+wRSaQ7dI= | JgsgCaCTqIaLeVhyL6XlRu3n7Rfk2FxMeK+wRSaQ7dI= | JgsgCaCTqIaLeVhyL6XlRu3n7Rfk2FxMeK+wRSaQ7dI= | 100     | 102     | 1234   | devnet-v33.0 | X4Bl4wQ9rCo= | advice pudding treat near rule blouse same whisper inner electric quit surface sunny dismiss leader blood seat clown cost exist hospital century reform able sponsor | gqNzaWfEQCzedNPqrhDsDBMY9vktJhaXjbJ2MJqr6Opt7Xae1uNQPVczCPin3feuk9YqhmmmLVNbzOgAS6nRh+K8MhgF+AKjdHhujaNmZWXNBQyiZnbNMLKjZ2VurGRldm5ldC12MzMuMKJnaMQgJgsgCaCTqIaLeVhyL6XlRu3n7Rfk2FxMeK+wRSaQ7dKibHbNNJqkbm90ZcQIX4Bl4wQ9rCqmc2Vsa2V5xCAmCyAJoJOohot5WHIvpeVG7eftF+TYXEx4r7BFJpDt0qNzbmTEIOfw+E0GgR358xyNh4sRVfRnHVGhhcIAkIZn9ElYcGihpHR5cGWma2V5cmVnp3ZvdGVmc3RkpnZvdGVrZM0E0qd2b3Rla2V5xCAmCyAJoJOohot5WHIvpeVG7eftF+TYXEx4r7BFJpDt0qd2b3RlbHN0Zg== |
    | 5   | 19    | 345   | JgsgCaCTqIaLeVhyL6XlRu3n7Rfk2FxMeK+wRSaQ7dI= | oImqaSLjuZj63/bNSAjd+eAh5JROOJ6j1cY4eGaJGX4= | uw62NBVKGAtqJ03XdSlcNtO6eq5rXbDMEMVGLbDzMN8= | 123     | 1000    | 65     | none         | none         | advice pudding treat near rule blouse same whisper inner electric quit surface sunny dismiss leader blood seat clown cost exist hospital century reform able sponsor | gqNzaWfEQNi9HHxsOAUwjXOHgpsEJvpe+b6hjLjIoWM9P389HMZ4RWZSB8uVvk1Kg+e52NvxTI3Vm9+Vl9W+dATm3m55ZAOjdHhui6NmZWXNBaWiZnYTomdoxCAmCyAJoJOohot5WHIvpeVG7eftF+TYXEx4r7BFJpDt0qJsds0BWaZzZWxrZXnEILsOtjQVShgLaidN13UpXDbTunqua12wzBDFRi2w8zDfo3NuZMQg5/D4TQaBHfnzHI2HixFV9GcdUaGFwgCQhmf0SVhwaKGkdHlwZaZrZXlyZWendm90ZWZzdHumdm90ZWtkQad2b3Rla2V5xCCgiappIuO5mPrf9s1ICN354CHklE44nqPVxjh4ZokZfqd2b3RlbHN0zQPo                                                 |
