/**
 * Type-safe GraphQL client with error handling and request/response logging
 */

import { config } from './config';
import type { GraphQLResponse } from '@/types';

export class GraphQLError extends Error {
  constructor(
    message: string,
    public errors?: Array<{ message: string; path?: string[] }>,
    public statusCode?: number
  ) {
    super(message);
    this.name = 'GraphQLError';
  }
}

export class NetworkError extends Error {
  constructor(message: string, public originalError?: Error) {
    super(message);
    this.name = 'NetworkError';
  }
}

type ApplicationEndpoint = 'factory' | 'token' | 'swap';

interface RequestOptions {
  signal?: AbortSignal;
  headers?: Record<string, string>;
  endpoint?: ApplicationEndpoint;
}

class GraphQLClient {
  private endpoints: Record<ApplicationEndpoint, string>;

  constructor() {
    this.endpoints = {
      factory: config.factoryEndpoint,
      token: config.tokenEndpoint,
      swap: config.swapEndpoint,
    };
  }

  private getEndpoint(endpoint?: ApplicationEndpoint): string {
    // Default to factory endpoint for token-related queries
    return endpoint ? this.endpoints[endpoint] : this.endpoints.factory;
  }

  async query<T>(
    query: string,
    variables?: Record<string, unknown>,
    options?: RequestOptions
  ): Promise<T> {
    const endpoint = this.getEndpoint(options?.endpoint);

    try {
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...options?.headers,
        },
        body: JSON.stringify({
          query,
          variables,
        }),
        signal: options?.signal,
      });

      if (!response.ok) {
        throw new NetworkError(
          `HTTP ${response.status}: ${response.statusText}`,
        );
      }

      const result: GraphQLResponse<T> = await response.json();

      if (result.errors && result.errors.length > 0) {
        console.error('GraphQL Errors:', result.errors);
        throw new GraphQLError(
          result.errors[0].message,
          result.errors,
          response.status
        );
      }

      if (!result.data) {
        throw new GraphQLError('No data returned from GraphQL query');
      }

      return result.data;
    } catch (error) {
      // NO MOCK DATA - Connect to REAL blockchain only
      if (error instanceof TypeError && error.message.includes('fetch')) {
        throw new NetworkError(
          'GraphQL endpoint not available. Please ensure Linera service is running on http://localhost:8080'
        );
      }

      if (error instanceof GraphQLError || error instanceof NetworkError) {
        throw error;
      }

      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          throw new NetworkError('Request was cancelled', error);
        }
        throw new NetworkError(`Network request failed: ${error.message}`, error);
      }

      throw new NetworkError('Unknown error occurred during GraphQL request');
    }
  }

  async mutate<T>(
    mutation: string,
    variables?: Record<string, unknown>,
    options?: RequestOptions
  ): Promise<T> {
    return this.query<T>(mutation, variables, options);
  }
}

export const graphqlClient = new GraphQLClient();

/**
 * Helper function to build GraphQL queries with type safety
 */
export const gql = (strings: TemplateStringsArray, ...values: unknown[]): string => {
  return strings.reduce((result, str, i) => {
    return result + str + (values[i] || '');
  }, '');
};
