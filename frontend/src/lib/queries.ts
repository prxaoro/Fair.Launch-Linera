/**
 * GraphQL query definitions
 */

import { gql } from './graphql-client';

export const TOKENS_QUERY = gql`
  query GetTokens($limit: Int, $offset: Int) {
    tokens(limit: $limit, offset: $offset) {
      tokenId
      creator
      metadata {
        name
        symbol
        description
        imageUrl
        twitter
        telegram
        website
      }
      curveConfig {
        k
        scale
        targetRaise
        maxSupply
      }
      currentSupply
      totalRaised
      isGraduated
      createdAt
      dexPoolId
    }
  }
`;

export const TOKEN_DETAIL_QUERY = gql`
  query GetTokenDetail($tokenId: String!) {
    token(tokenId: $tokenId) {
      tokenId
      creator
      metadata {
        name
        symbol
        description
        imageUrl
        twitter
        telegram
        website
      }
      curveConfig {
        k
        scale
        targetRaise
        maxSupply
      }
      currentSupply
      totalRaised
      isGraduated
      createdAt
      dexPoolId
    }
  }
`;

export const TOKEN_INFO_QUERY = gql`
  query GetTokenInfo {
    token_info {
      token_id
      creator
      name
      symbol
      description
      current_supply
      total_raised
      current_price
      holder_count
      trade_count
      is_graduated
      progress_percentage
    }
  }
`;

export const RECENT_TRADES_QUERY = gql`
  query GetRecentTrades($limit: Int) {
    recent_trades(limit: $limit) {
      token_id
      is_buy
      token_amount
      currency_amount
      price
    }
  }
`;

export const PORTFOLIO_QUERY = gql`
  query GetPortfolio($accountJson: String!) {
    balance(account_json: $accountJson)
    user_position(account_json: $accountJson) {
      token_id
      balance
      total_invested
      trades_count
    }
    user_trades(account_json: $accountJson, limit: 100) {
      token_id
      is_buy
      token_amount
      currency_amount
      price
    }
  }
`;

export const BUY_QUOTE_QUERY = gql`
  query GetBuyQuote($amount: String!) {
    buy_quote(amount: $amount) {
      token_amount
      currency_amount
      price_impact
      new_price
    }
  }
`;

export const SELL_QUOTE_QUERY = gql`
  query GetSellQuote($amount: String!) {
    sell_quote(amount: $amount) {
      token_amount
      currency_amount
      price_impact
      new_price
    }
  }
`;

export const STATS_QUERY = gql`
  query GetStats {
    stats {
      totalTokens
      graduatedCount
      activeCount
      totalValueLocked
    }
  }
`;

export const ALLOWANCE_QUERY = gql`
  query GetAllowance($ownerJson: String!, $spenderJson: String!) {
    allowance(owner_json: $ownerJson, spender_json: $spenderJson)
  }
`;
