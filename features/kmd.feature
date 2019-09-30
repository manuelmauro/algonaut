Feature: KMD
  Background:
    Given a kmd client
    And wallet information

  Scenario: Version
    When I get versions with kmd
    Then v1 should be in the versions

  Scenario: Create and rename wallet
    When I create a wallet
    Then the wallet should exist
    When I get the wallet handle
    Then I can get the master derivation key
    When I rename the wallet
    Then I can still get the wallet information with the same handle

  Scenario: Wallet handle
    When I get the wallet handle
    And I renew the wallet handle
    And I release the wallet handle
    Then the wallet handle should not work

  Scenario: Generate and delete key
    When I generate a key using kmd
    Then the key should be in the wallet
    When I delete the key
    Then the key should not be in the wallet

  Scenario: Make account and get info
    Given an algod client
    When I generate a key using kmd
    Then I can get account information

  Scenario: Import and export key
    When I generate a key
    And I import the key
    Then the private key should be equal to the exported private key

  Scenario Outline: Sign both ways
    Given an algod client
    And default transaction with parameters <amt> "<note>"
    When I get the private key
    And I sign the transaction with the private key
    And I sign the transaction with kmd
    Then the signed transaction should equal the kmd signed transaction

    Examples:
    | amt | note |
    | 0   | X4Bl4wQ9rCo= |
    | 1234523 | X4Bl4wQ9rCo= |

  Scenario Outline: Import and export multisig
    Given multisig addresses "<addresses>"
    When I import the multisig
    Then the multisig should be in the wallet
    When I export the multisig
    Then the multisig should equal the exported multisig
    When I delete the multisig
    Then the multisig should not be in the wallet

    Examples:
    | addresses |
    | DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA BFRTECKTOOE7A5LHCF3TTEOH2A7BW46IYT2SX5VP6ANKEXHZYJY77SJTVM 47YPQTIGQEO7T4Y4RWDYWEKV6RTR2UNBQXBABEEGM72ESWDQNCQ52OPASU |

  Scenario Outline: Sign multisig both ways
    Given an algod client
    And default multisig transaction with parameters <amt> "<note>"
    When I sign the multisig transaction with kmd
    And I get the private key
    And I sign the multisig transaction with the private key
    Then the multisig transaction should equal the kmd signed multisig transaction

    Examples:
    | amt | note |
    | 0   | X4Bl4wQ9rCo= |
    | 1234523 | X4Bl4wQ9rCo= |
