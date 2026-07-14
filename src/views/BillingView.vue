<template>
  <div class="billing-view">
    <!-- Header -->
    <div class="page-header">
      <div>
        <h1>Billing</h1>
        <p class="text-secondary text-sm">Cost analysis and usage breakdown</p>
      </div>
      <div class="page-actions">
        <span v-if="cacheActive" class="cache-indicator" title="Data from cache">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
            <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.3"/>
            <path d="M8 4V8L10.5 9.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          {{ cacheRemainingLabel }}
        </span>
        <button class="btn-secondary" @click="forceRefresh" :disabled="loading || !accountsStore.activeAccountId">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" :class="{ spinning: loading }">
            <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <path d="M14 2V6H10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Refresh
        </button>
      </div>
    </div>

    <!-- No Account State -->
    <div v-if="!accountsStore.activeAccountId" class="empty-state">
      <div class="empty-icon">
        <svg width="40" height="40" viewBox="0 0 16 16" fill="none">
          <path d="M8 1V15M4 4H12M3 7H13M4 10H12M5 13H11" stroke="currentColor" stroke-width="1" stroke-linecap="round"/>
        </svg>
      </div>
      <p class="text-secondary">Select an AWS account from the sidebar to view billing data.</p>
    </div>

    <!-- Date Filter + Content -->
    <template v-else>
      <!-- Period Filter Bar -->
      <div class="period-filter">
        <div class="period-presets">
          <button 
            v-for="preset in periodPresets" 
            :key="preset.id" 
            class="period-btn" 
            :class="{ 'period-btn--active': activePeriod === preset.id }"
            @click="selectPeriod(preset.id)"
          >{{ preset.label }}</button>
        </div>
        <div v-if="activePeriod === 'custom'" class="period-custom">
          <input type="month" v-model="customStart" class="glass-input period-input" />
          <span class="text-tertiary">→</span>
          <input type="month" v-model="customEnd" class="glass-input period-input" />
          <button class="btn-secondary" @click="applyCustomRange" :disabled="!customStart || !customEnd">Apply</button>
        </div>
        <div v-if="reportGranularity" class="period-info">
          <span class="text-tertiary text-sm">{{ reportDateRange }} · {{ reportGranularity }} granularity</span>
        </div>
      </div>

      <!-- Error -->
      <div v-if="errorMsg && !loading" class="error-banner">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.3"/>
          <path d="M8 5V9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          <circle cx="8" cy="11.5" r="0.75" fill="currentColor"/>
        </svg>
        <span>{{ errorMsg }}</span>
      </div>
      <!-- Summary Cards -->
      <div class="summary-cards">
        <div class="summary-card">
          <div class="summary-label">Current Month</div>
          <div class="summary-value" :class="{ 'skeleton-text': loading }">
            <template v-if="!loading">{{ formatCurrency(summary.currentMonthTotal) }}</template>
          </div>
          <div class="summary-sub text-tertiary text-sm">
            <template v-if="!loading && summary.lastMonthTotal">
              <span :class="monthDeltaClass">{{ monthDeltaLabel }}</span> vs last month
            </template>
          </div>
        </div>

        <div class="summary-card">
          <div class="summary-label">Last Month</div>
          <div class="summary-value" :class="{ 'skeleton-text': loading }">
            <template v-if="!loading">{{ formatCurrency(summary.lastMonthTotal) }}</template>
          </div>
          <div class="summary-sub text-tertiary text-sm">
            <template v-if="!loading">Final total</template>
          </div>
        </div>

        <div class="summary-card">
          <div class="summary-label">Forecast (EOM)</div>
          <div class="summary-value" :class="{ 'skeleton-text': loading }">
            <template v-if="!loading">{{ formatCurrency(summary.forecastMonthTotal) }}</template>
          </div>
          <div class="summary-sub text-tertiary text-sm">
            <template v-if="!loading">Projected end of month</template>
          </div>
        </div>
      </div>

      <!-- Main Content Grid -->
      <div class="billing-grid">
        <!-- Cost Chart (from report) -->
        <div class="billing-panel">
          <div class="panel-header">
            <h3>Cost Over Time</h3>
            <span class="text-tertiary text-sm">{{ reportGranularity || 'Daily' }}</span>
          </div>
          <div class="chart-container" v-if="!loadingReport && reportDataPoints.length > 0">
            <div class="bar-chart">
              <div 
                v-for="point in reportDataPoints" 
                :key="point.date" 
                class="bar-col"
                :title="`${point.date}: ${formatCurrency(point.amount)}`"
              >
                <div class="bar" :style="{ height: barHeightReport(point.amount) }"></div>
                <div class="bar-label">{{ chartLabel(point.date) }}</div>
              </div>
            </div>
          </div>
          <div v-else-if="loading || loadingReport" class="chart-skeleton">
            <div v-for="i in 15" :key="i" class="skeleton-bar" :style="{ height: `${20 + Math.random() * 60}%` }"></div>
          </div>
          <div v-else class="empty-chart">
            <p class="text-tertiary text-sm">No cost data for this period</p>
          </div>
        </div>

        <!-- Top Services -->
        <div class="billing-panel">
          <div class="panel-header">
            <h3>Top Services</h3>
            <span class="text-tertiary text-sm">by cost</span>
          </div>
          <div v-if="!loading && summary.services && summary.services.length > 0" class="services-list">
            <div v-for="svc in topServices" :key="svc.service" class="service-row">
              <div class="service-info">
                <span class="service-name">{{ shortServiceName(svc.service) }}</span>
                <span class="service-amount">{{ formatCurrency(svc.amount) }}</span>
              </div>
              <div class="service-bar-bg">
                <div class="service-bar-fill" :style="{ width: serviceBarWidth(svc.amount) }"></div>
              </div>
            </div>
          </div>
          <div v-else-if="loading" class="services-skeleton">
            <div v-for="i in 5" :key="i" class="service-skeleton-row">
              <div class="skeleton" style="width: 55%; height: 12px;"></div>
              <div class="skeleton" style="width: 100%; height: 6px; margin-top: 8px; border-radius: 3px;"></div>
            </div>
          </div>
          <div v-else class="empty-chart">
            <p class="text-tertiary text-sm">No service cost data available</p>
          </div>
        </div>
      </div>

      <!-- All Services Table -->
      <div class="billing-panel" v-if="!loading && summary.services && summary.services.length > 0" style="margin-top: 1rem;">
        <div class="panel-header">
          <h3>All Services</h3>
          <span class="badge badge--neutral">{{ summary.services.length }}</span>
        </div>
        <div class="services-table">
          <div class="table-header">
            <span>Service</span>
            <span>Cost</span>
          </div>
          <div v-for="svc in summary.services" :key="svc.service" class="table-row">
            <span class="table-service">{{ svc.service }}</span>
            <span class="table-amount">{{ formatCurrency(svc.amount) }} <span class="text-tertiary">{{ svc.unit }}</span></span>
          </div>
        </div>
      </div>

      <!-- Usage by Service -->
      <div class="billing-panel" style="margin-top: 1rem;">
        <div class="panel-header">
          <h3>Resource Usage</h3>
          <span class="text-tertiary text-sm">current month — all services (incl. free tier)</span>
        </div>

        <div v-if="loadingUsage" class="services-skeleton">
          <div v-for="i in 4" :key="i" class="service-skeleton-row">
            <div class="skeleton" style="width: 55%; height: 12px;"></div>
            <div class="skeleton" style="width: 100%; height: 6px; margin-top: 8px; border-radius: 3px;"></div>
          </div>
        </div>

        <div v-else-if="usageError" class="error-banner" style="margin-bottom: 0;">
          <span>{{ usageError }}</span>
        </div>

        <div v-else-if="usageServices.length === 0" class="empty-chart" style="height: auto; padding: 2rem;">
          <p class="text-tertiary text-sm">No usage data available for this period.</p>
        </div>

        <div v-else class="usage-services">
          <div 
            v-for="svc in usageServices" 
            :key="svc.service" 
            class="usage-service-card"
            :class="{ 'usage-service-card--expanded': expandedServices.has(svc.service) }"
          >
            <div class="usage-service-header" @click="toggleServiceExpand(svc.service)">
              <div class="usage-service-title">
                <svg width="12" height="12" viewBox="0 0 16 16" fill="none" class="expand-icon" :class="{ 'expand-icon--open': expandedServices.has(svc.service) }">
                  <path d="M6 4L10 8L6 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
                <span class="usage-service-name">{{ shortServiceName(svc.service) }}</span>
              </div>
              <div class="usage-service-meta">
                <span class="badge badge--neutral">{{ svc.usage_details.length }} types</span>
                <span class="usage-service-cost">{{ formatCurrency(svc.total_cost) }}</span>
              </div>
            </div>
            <div v-if="expandedServices.has(svc.service)" class="usage-details-list">
              <div class="usage-details-header">
                <span>Usage Type</span>
                <span>Usage</span>
                <span>Rate</span>
                <span>Cost</span>
              </div>
              <div v-for="detail in svc.usage_details" :key="detail.usage_type" class="usage-detail-row">
                <span class="usage-detail-type">
                  {{ formatUsageType(detail.usage_type) }}
                  <span v-if="detail.is_free_tier" class="free-tier-badge">FREE</span>
                </span>
                <span class="usage-detail-amount">{{ formatUsageAmount(detail.amount) }} <span class="text-tertiary">{{ detail.unit }}</span></span>
                <span class="usage-detail-rate">{{ formatUnitPrice(detail.unit_price, detail.unit) }}</span>
                <span class="usage-detail-cost" :class="{ 'usage-detail-cost--free': detail.is_free_tier }">{{ formatCurrency(detail.cost) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useAwsAccountsStore } from '../store/awsAccounts';
import { useBillingStore, type CostSummary, type ServiceUsage } from '../store/billing';
import { invoke } from '@tauri-apps/api/core';

const accountsStore = useAwsAccountsStore();
const billingStore = useBillingStore();

// Local UI state
const expandedServices = ref<Set<string>>(new Set());

// Period filter
type PeriodId = '7d' | '1m' | '3m' | '6m' | '1y' | 'custom';
const activePeriod = ref<PeriodId>('1m');
const customStart = ref('');
const customEnd = ref('');

const periodPresets = [
  { id: '7d' as PeriodId, label: '7 days' },
  { id: '1m' as PeriodId, label: '1 month' },
  { id: '3m' as PeriodId, label: '3 months' },
  { id: '6m' as PeriodId, label: '6 months' },
  { id: '1y' as PeriodId, label: '1 year' },
  { id: 'custom' as PeriodId, label: 'Custom' },
];

// Report state
interface ReportDataPoint {
  date: string;
  amount: string;
  unit: string;
}

const loadingReport = ref(false);
const reportDataPoints = ref<ReportDataPoint[]>([]);
const reportServices = ref<{ service: string; amount: string; unit: string }[]>([]);
const reportGranularity = ref('');
const reportTotalAmount = ref('');
const reportDateRange = ref('');

// Reactive computed from store
const loading = computed(() => billingStore.loadingCost);
const loadingUsage = computed(() => billingStore.loadingUsage);
const errorMsg = computed(() => billingStore.errorCost);
const usageError = computed(() => billingStore.errorUsage);

const summary = computed<CostSummary>(() => {
  const id = accountsStore.activeAccountId;
  if (!id) return emptySummary();
  const cached = billingStore.getCacheForAccount(id);
  return cached?.costSummary ?? emptySummary();
});

const usageServices = computed<ServiceUsage[]>(() => {
  const id = accountsStore.activeAccountId;
  if (!id) return [];
  const cached = billingStore.getCacheForAccount(id);
  return cached?.usageServices ?? [];
});

const cacheActive = computed(() => {
  const id = accountsStore.activeAccountId;
  if (!id) return false;
  return billingStore.isCacheValid(id);
});

const cacheRemainingLabel = computed(() => {
  const id = accountsStore.activeAccountId;
  if (!id) return '';
  const ms = billingStore.getCacheTTLRemaining(id);
  if (ms <= 0) return '';
  const min = Math.floor(ms / 60000);
  if (min >= 60) {
    const h = Math.floor(min / 60);
    const m = min % 60;
    return `${h}h ${m}m`;
  }
  return `${min}m`;
});

function emptySummary(): CostSummary {
  return {
    currentMonthTotal: null,
    lastMonthTotal: null,
    forecastMonthTotal: null,
    unit: null,
    services: null,
    dailyCosts: null,
  };
}

// Computed
const topServices = computed(() => {
  if (!summary.value.services) return [];
  return summary.value.services.slice(0, 7);
});

const maxServiceCost = computed(() => {
  if (!summary.value.services || summary.value.services.length === 0) return 1;
  return parseFloat(summary.value.services[0].amount) || 1;
});

const monthDelta = computed(() => {
  const current = parseFloat(summary.value.currentMonthTotal || '0');
  const last = parseFloat(summary.value.lastMonthTotal || '0');
  if (last === 0) return 0;
  return ((current - last) / last) * 100;
});

const monthDeltaLabel = computed(() => {
  const d = monthDelta.value;
  if (d === 0) return '—';
  const sign = d > 0 ? '+' : '';
  return `${sign}${d.toFixed(0)}%`;
});

const monthDeltaClass = computed(() => {
  if (monthDelta.value > 10) return 'delta-up';
  if (monthDelta.value < -10) return 'delta-down';
  return 'delta-neutral';
});

// Methods
function formatCurrency(value: string | null | undefined): string {
  if (!value) return '$0.00';
  const num = parseFloat(value);
  if (isNaN(num)) return '$0.00';
  return `$${num.toFixed(2)}`;
}

function serviceBarWidth(amount: string): string {
  const val = parseFloat(amount) || 0;
  const pct = (val / maxServiceCost.value) * 100;
  return `${Math.max(pct, 1)}%`;
}

function shortServiceName(name: string): string {
  return name
    .replace('Amazon ', '')
    .replace('AWS ', '')
    .replace('Amazon', '')
    .replace('AWS', '')
    .trim();
}

function toggleServiceExpand(service: string) {
  const s = new Set(expandedServices.value);
  if (s.has(service)) {
    s.delete(service);
  } else {
    s.add(service);
  }
  expandedServices.value = s;
}

function formatUsageType(usageType: string): string {
  return usageType.replace(/^[A-Z]{2,4}\d?-/, '');
}

function formatUsageAmount(amount: string): string {
  const num = parseFloat(amount);
  if (isNaN(num)) return '0';
  if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(2)}M`;
  if (num >= 1_000) return `${(num / 1_000).toFixed(2)}K`;
  if (num < 0.01) return num.toExponential(2);
  return num.toFixed(2);
}

function formatUnitPrice(price: string, unit: string): string {
  const num = parseFloat(price);
  if (isNaN(num) || num === 0) return '—';
  if (num < 0.0000001) return `$${num.toExponential(2)}/${unit}`;
  if (num < 0.001) return `$${num.toFixed(7)}/${unit}`;
  if (num < 1) return `$${num.toFixed(4)}/${unit}`;
  return `$${num.toFixed(2)}/${unit}`;
}

function getAccountCreds() {
  const activeId = accountsStore.activeAccountId;
  if (!activeId) return null;
  const acc = accountsStore.accounts.find(a => a.id === activeId);
  if (!acc) return null;
  return { accountId: activeId, accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region };
}

// Period helpers
function getDateRange(period: PeriodId): { start: string; end: string } {
  const now = new Date();
  const end = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
  
  let startDate: Date;
  switch (period) {
    case '7d':
      startDate = new Date(now.getTime() - 7 * 86400000);
      break;
    case '1m':
      startDate = new Date(now.getFullYear(), now.getMonth(), 1);
      break;
    case '3m':
      startDate = new Date(now.getFullYear(), now.getMonth() - 2, 1);
      break;
    case '6m':
      startDate = new Date(now.getFullYear(), now.getMonth() - 5, 1);
      break;
    case '1y':
      startDate = new Date(now.getFullYear() - 1, now.getMonth(), 1);
      break;
    default:
      startDate = new Date(now.getFullYear(), now.getMonth(), 1);
  }
  
  const start = `${startDate.getFullYear()}-${String(startDate.getMonth() + 1).padStart(2, '0')}-${String(startDate.getDate()).padStart(2, '0')}`;
  return { start, end };
}

function selectPeriod(id: PeriodId) {
  activePeriod.value = id;
  if (id !== 'custom') {
    fetchReport();
  }
}

function applyCustomRange() {
  if (!customStart.value || !customEnd.value) return;
  fetchReport();
}

async function fetchReport() {
  const creds = getAccountCreds();
  if (!creds) return;

  let start: string, end: string;

  if (activePeriod.value === 'custom') {
    start = customStart.value + '-01';
    // End of the selected month
    const [y, m] = customEnd.value.split('-').map(Number);
    const nextMonth = new Date(y, m, 1); // m is already 1-indexed from split
    end = `${nextMonth.getFullYear()}-${String(nextMonth.getMonth() + 1).padStart(2, '0')}-${String(nextMonth.getDate()).padStart(2, '0')}`;
  } else {
    const range = getDateRange(activePeriod.value);
    start = range.start;
    end = range.end;
  }

  loadingReport.value = true;

  try {
    const res: any = await invoke('get_cost_report', {
      accessKeyId: creds.accessKeyId,
      secretAccessKey: creds.secretAccessKey,
      region: creds.region,
      startDate: start,
      endDate: end,
    });

    if (res.success) {
      reportDataPoints.value = res.data_points || [];
      reportServices.value = res.services || [];
      reportGranularity.value = res.granularity === 'DAILY' ? 'Daily' : 'Monthly';
      reportTotalAmount.value = res.total_amount || '0';
      reportDateRange.value = `${start} → ${end}`;
    }
  } catch (e: any) {
    // Silent fail for report, main data still shows
  } finally {
    loadingReport.value = false;
  }
}

// Chart helpers for report
const maxReportCost = computed(() => {
  if (reportDataPoints.value.length === 0) return 1;
  const max = Math.max(...reportDataPoints.value.map(d => parseFloat(d.amount) || 0));
  return max > 0 ? max : 1;
});

function barHeightReport(amount: string): string {
  const val = parseFloat(amount) || 0;
  const pct = (val / maxReportCost.value) * 100;
  return `${Math.max(pct, 2)}%`;
}

function chartLabel(date: string): string {
  if (!date) return '';
  const d = new Date(date + 'T00:00:00');
  if (reportGranularity.value === 'Monthly') {
    return d.toLocaleDateString('en-US', { month: 'short' });
  }
  return d.getDate().toString();
}

async function fetchAll(force: boolean = false) {
  const creds = getAccountCreds();
  if (!creds) return;

  await billingStore.fetchAll(
    creds.accountId,
    creds.accessKeyId,
    creds.secretAccessKey,
    creds.region,
    force,
  );

  // Auto-expand first 3 usage services
  if (usageServices.value.length > 0 && expandedServices.value.size === 0) {
    expandedServices.value = new Set(
      usageServices.value.slice(0, 3).map(s => s.service)
    );
  }

  // Fetch report for chart
  fetchReport();
}

function forceRefresh() {
  fetchAll(true);
}

onMounted(() => {
  if (accountsStore.activeAccountId) {
    fetchAll(false);
  }
});

watch(() => accountsStore.activeAccountId, (newId) => {
  if (newId) {
    expandedServices.value = new Set();
    fetchAll(false); // will use cache if valid
  }
});
</script>

<style scoped>
.billing-view {
  max-width: 1024px;
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 1.5rem;
}

.page-header h1 {
  margin-bottom: 0.25rem;
}

.page-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

/* Cache Indicator */
.cache-indicator {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.6875rem;
  color: var(--text-tertiary);
  padding: 0.25rem 0.625rem;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  font-variant-numeric: tabular-nums;
}

/* Empty / Error */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 4rem 2rem;
  gap: 1rem;
}

.empty-icon {
  color: var(--text-tertiary);
}

.error-banner {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 0.875rem;
  background: var(--error-subtle);
  border: 1px solid rgba(238, 0, 0, 0.15);
  border-radius: var(--radius-sm);
  color: #ff6166;
  font-size: 0.8125rem;
}

/* Period Filter */
.period-filter {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 1.25rem;
  padding: 0.625rem 0.875rem;
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
}

.period-presets {
  display: flex;
  gap: 2px;
  background: var(--bg-elevated);
  border-radius: var(--radius-sm);
  padding: 2px;
}

.period-btn {
  padding: 0.375rem 0.75rem;
  background: none;
  border: none;
  border-radius: 4px;
  color: var(--text-tertiary);
  font-size: 0.75rem;
  font-weight: 450;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.period-btn:hover {
  color: var(--text-secondary);
}

.period-btn--active {
  background: var(--bg-primary);
  color: var(--text-primary);
  font-weight: 500;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.period-custom {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.period-input {
  width: 130px;
  font-size: 0.75rem;
  padding: 0.3rem 0.5rem;
}

.period-info {
  margin-left: auto;
}

/* Summary Cards */
.summary-cards {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.75rem;
  margin-bottom: 1.5rem;
}

.summary-card {
  padding: 1.25rem;
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.summary-label {
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-tertiary);
  font-weight: 500;
}

.summary-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.summary-sub {
  margin-top: 0.25rem;
}

.skeleton-text {
  background: var(--bg-elevated);
  border-radius: 4px;
  width: 80px;
  height: 1.5rem;
  animation: pulse 1.5s ease infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* Delta */
.delta-up {
  color: #ef4444;
  font-weight: 500;
}

.delta-down {
  color: #22c55e;
  font-weight: 500;
}

.delta-neutral {
  color: var(--text-tertiary);
}

/* Grid */
.billing-grid {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 0.75rem;
}

/* Panel */
.billing-panel {
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  padding: 1.25rem;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.panel-header h3 {
  margin: 0;
  font-size: 0.875rem;
}

/* Bar Chart */
.chart-container {
  height: 180px;
}

.bar-chart {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  height: 100%;
  padding-bottom: 20px;
  position: relative;
}

.bar-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  height: 100%;
  position: relative;
  cursor: default;
}

.bar {
  width: 100%;
  max-width: 24px;
  background: var(--accent);
  border-radius: 3px 3px 0 0;
  transition: height 0.3s ease, opacity var(--transition-fast);
  min-height: 2px;
  opacity: 0.75;
}

.bar-col:hover .bar {
  opacity: 1;
}

.bar-label {
  font-size: 0.5625rem;
  color: var(--text-tertiary);
  margin-top: 4px;
  position: absolute;
  bottom: 0;
}

.chart-skeleton {
  height: 180px;
  display: flex;
  align-items: flex-end;
  gap: 4px;
  padding-bottom: 20px;
}

.skeleton-bar {
  flex: 1;
  background: var(--bg-elevated);
  border-radius: 3px 3px 0 0;
  animation: pulse 1.5s ease infinite;
}

.empty-chart {
  height: 180px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Services */
.services-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.service-row {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.service-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.service-name {
  font-size: 0.75rem;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 65%;
}

.service-amount {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.service-bar-bg {
  height: 4px;
  background: var(--bg-elevated);
  border-radius: 2px;
  overflow: hidden;
}

.service-bar-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.4s ease;
}

.services-skeleton {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.service-skeleton-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

/* Table */
.services-table {
  display: flex;
  flex-direction: column;
  max-height: 300px;
  overflow-y: auto;
}

.table-header {
  display: flex;
  justify-content: space-between;
  padding: 0.5rem 0;
  border-bottom: 1px solid var(--border-default);
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-tertiary);
  font-weight: 500;
  position: sticky;
  top: 0;
  background: var(--bg-primary);
}

.table-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0;
  border-bottom: 1px solid var(--border-default);
  transition: background var(--transition-fast);
}

.table-row:last-child {
  border-bottom: none;
}

.table-row:hover {
  background: var(--bg-hover);
  margin: 0 -0.5rem;
  padding: 0.5rem;
  border-radius: var(--radius-sm);
}

.table-service {
  font-size: 0.8125rem;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 70%;
}

.table-amount {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

/* Utilities */
.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Usage Section */
.usage-services {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.usage-service-card {
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  overflow: hidden;
  transition: all var(--transition-fast);
}

.usage-service-card:hover {
  border-color: var(--border-hover);
}

.usage-service-card--expanded {
  border-color: var(--border-hover);
  background: var(--bg-secondary);
}

.usage-service-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 0.875rem;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.usage-service-header:hover {
  background: var(--bg-hover);
}

.usage-service-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.expand-icon {
  transition: transform var(--transition-fast);
  color: var(--text-tertiary);
}

.expand-icon--open {
  transform: rotate(90deg);
}

.usage-service-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
}

.usage-service-meta {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.usage-service-cost {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.usage-details-list {
  border-top: 1px solid var(--border-default);
  padding: 0.5rem 0.875rem 0.625rem;
  display: flex;
  flex-direction: column;
  gap: 0;
  animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.usage-details-header {
  display: grid;
  grid-template-columns: 1.5fr 1fr 1fr 0.7fr;
  gap: 0.5rem;
  padding: 0.375rem 0 0.375rem 1.25rem;
  font-size: 0.625rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-tertiary);
  font-weight: 500;
  border-bottom: 1px solid var(--border-default);
}

.usage-detail-row {
  display: grid;
  grid-template-columns: 1.5fr 1fr 1fr 0.7fr;
  gap: 0.5rem;
  align-items: center;
  padding: 0.375rem 0 0.375rem 1.25rem;
  border-bottom: 1px solid var(--border-default);
}

.usage-detail-row:last-child {
  border-bottom: none;
}

.usage-detail-type {
  font-size: 0.75rem;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.free-tier-badge {
  font-size: 0.5625rem;
  font-weight: 600;
  color: #22c55e;
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.2);
  padding: 0.0625rem 0.3125rem;
  border-radius: 3px;
  font-family: -apple-system, sans-serif;
  letter-spacing: 0.02em;
}

.usage-detail-amount {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.usage-detail-rate {
  font-size: 0.6875rem;
  color: var(--text-tertiary);
  font-family: 'JetBrains Mono', monospace;
  white-space: nowrap;
}

.usage-detail-cost {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  text-align: right;
}

.usage-detail-cost--free {
  color: #22c55e;
}
</style>
