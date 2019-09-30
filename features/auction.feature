Feature: Auction
  Scenario: Encode and decode a bid
    When I create a bid
    And I sign the bid
    And I encode and decode the bid
    Then the bid should still be the same
