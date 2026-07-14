import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

// Cache TTL: 2 hours
const CACHE_TTL_MS = 2 * 60 * 60 * 1000;

export interface ServiceCost {
  service: string;
  amount: string;
  unit: string;
}

export interface DailyCost {
  date: string;
  amount: string;
  unit: string;
}

export interface UsageDetail {
  usage_type: string;
  amount: string;
  unit: string;
  cost: string;
  cost_unit: string;
  unit_price: string;
  is_free_tier: boolean;
}

export interface ServiceUsage {
  service: string;
  usage_details: UsageDetail[];
  total_usage: string;
  total_cost: string;
}

export interface CostSummary {
  currentMonthTotal: string | null;
  lastMonthTotal: string | null;
  forecastMonthTotal: string | null;
  unit: string | null;
  services: ServiceCost[] | null;
  dailyCosts: DailyCost[] | null;
}

interface AccountCache {
  costSummary: CostSummary;
  usageServices: ServiceUsage[];
  fetchedAt: number; // timestamp ms
}

interface BillingState {
  cache: Record<string, AccountCache>; // keyed by account ID
  loadingCost: boolean;
  loadingUsage: boolean;
  errorCost: string;
  errorUsage: string;
}

export const useBillingStore = defineStore('billing', {
  state: (): BillingState => ({
    cache: {},
    loadingCost: false,
    loadingUsage: false,
    errorCost: '',
    errorUsage: '',
  }),

  getters: {
    getCacheForAccount: (state) => (accountId: string): AccountCache | null => {
      const entry = state.cache[accountId];
      if (!entry) return null;
      const age = Date.now() - entry.fetchedAt;
      if (age > CACHE_TTL_MS) return null; // expired
      return entry;
    },

    getCacheTTLRemaining: (state) => (accountId: string): number => {
      const entry = state.cache[accountId];
      if (!entry) return 0;
      const remaining = CACHE_TTL_MS - (Date.now() - entry.fetchedAt);
      return Math.max(0, remaining);
    },

    isCacheValid: (state) => (accountId: string): boolean => {
      const entry = state.cache[accountId];
      if (!entry) return false;
      return (Date.now() - entry.fetchedAt) < CACHE_TTL_MS;
    },
  },

  actions: {
    async fetchCostSummary(
      accountId: string,
      accessKeyId: string,
      secretAccessKey: string,
      region: string,
      force: boolean = false,
    ): Promise<CostSummary | null> {
      // Check cache unless forced
      if (!force) {
        const cached = this.getCacheForAccount(accountId);
        if (cached) return cached.costSummary;
      }

      this.loadingCost = true;
      this.errorCost = '';

      try {
        const res: any = await invoke('get_cost_summary', {
          accessKeyId,
          secretAccessKey,
          region,
        });

        if (res.success) {
          const summary: CostSummary = {
            currentMonthTotal: res.current_month_total,
            lastMonthTotal: res.last_month_total,
            forecastMonthTotal: res.forecast_month_total,
            unit: res.unit,
            services: res.services,
            dailyCosts: res.daily_costs,
          };

          // Update cache
          if (!this.cache[accountId]) {
            this.cache[accountId] = {
              costSummary: summary,
              usageServices: [],
              fetchedAt: Date.now(),
            };
          } else {
            this.cache[accountId].costSummary = summary;
            this.cache[accountId].fetchedAt = Date.now();
          }

          return summary;
        } else {
          this.errorCost = res.error_message || 'Failed to fetch billing data.';
          return null;
        }
      } catch (err: any) {
        this.errorCost = err.message || err.toString();
        return null;
      } finally {
        this.loadingCost = false;
      }
    },

    async fetchUsage(
      accountId: string,
      accessKeyId: string,
      secretAccessKey: string,
      region: string,
      force: boolean = false,
    ): Promise<ServiceUsage[] | null> {
      // Check cache unless forced
      if (!force) {
        const cached = this.getCacheForAccount(accountId);
        if (cached && cached.usageServices.length > 0) return cached.usageServices;
      }

      this.loadingUsage = true;
      this.errorUsage = '';

      try {
        const res: any = await invoke('get_usage_by_service', {
          accessKeyId,
          secretAccessKey,
          region,
        });

        if (res.success) {
          const services: ServiceUsage[] = res.services || [];

          // Update cache
          if (!this.cache[accountId]) {
            this.cache[accountId] = {
              costSummary: {
                currentMonthTotal: null,
                lastMonthTotal: null,
                forecastMonthTotal: null,
                unit: null,
                services: null,
                dailyCosts: null,
              },
              usageServices: services,
              fetchedAt: Date.now(),
            };
          } else {
            this.cache[accountId].usageServices = services;
          }

          return services;
        } else {
          this.errorUsage = res.error_message || 'Failed to fetch usage data.';
          return null;
        }
      } catch (err: any) {
        this.errorUsage = err.message || err.toString();
        return null;
      } finally {
        this.loadingUsage = false;
      }
    },

    async fetchAll(
      accountId: string,
      accessKeyId: string,
      secretAccessKey: string,
      region: string,
      force: boolean = false,
    ) {
      await Promise.all([
        this.fetchCostSummary(accountId, accessKeyId, secretAccessKey, region, force),
        this.fetchUsage(accountId, accessKeyId, secretAccessKey, region, force),
      ]);
    },

    clearCache(accountId?: string) {
      if (accountId) {
        delete this.cache[accountId];
      } else {
        this.cache = {};
      }
    },
  },
});
