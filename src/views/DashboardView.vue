<template>
  <div class="dashboard-view">
    <!-- Header -->
    <div class="page-header">
      <div>
        <h1>Dashboard</h1>
        <p class="text-secondary text-sm">Overview of your AWS resources</p>
      </div>
      <div class="page-actions" v-if="accountsStore.activeAccountId">
        <button class="btn-secondary" @click="refreshAll" :disabled="loadingSummary">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" :class="{ spinning: loadingSummary }">
            <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <path d="M14 2V6H10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Refresh
        </button>
      </div>
    </div>

    <!-- Empty state: No accounts -->
    <div v-if="accountsStore.accounts.length === 0 && !showAddForm" class="empty-state">
      <div class="empty-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
          <path d="M6.5 20C4.98 20 3.68 19.47 2.61 18.41C1.54 17.35 1 16.07 1 14.55C1 13.22 1.4 12.04 2.19 11.03C2.98 10.02 4.01 9.37 5.25 9.08C5.63 7.57 6.44 6.33 7.69 5.38C8.94 4.42 10.38 3.95 12 3.95C14 3.95 15.72 4.68 17.14 6.15C18.57 7.62 19.28 9.37 19.28 11.42V11.95C20.47 12.03 21.46 12.47 22.25 13.27C23.04 14.07 23.44 15.03 23.44 16.15C23.44 17.22 23.06 18.14 22.31 18.89C21.56 19.64 20.65 20 19.58 20H6.5Z" stroke="currentColor" stroke-width="1.2" fill="none"/>
        </svg>
      </div>
      <h2>Welcome to Easy Cloud</h2>
      <p class="text-secondary">Connect your AWS account to get started.</p>
      <button class="btn-primary" @click="showAddForm = true">Add AWS Account</button>
    </div>

    <!-- Main Dashboard Content -->
    <template v-if="accountsStore.activeAccountId">
      <!-- Stat Cards -->
      <div class="stat-cards">
        <div class="stat-card" @click="$router.push('/s3')">
          <div class="stat-icon stat-icon--s3"><svg width="20" height="20" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3"><path d="M2 4C2 4 4 2 8 2C12 2 14 4 14 4V12C14 12 12 14 8 14C4 14 2 12 2 12V4Z"/><path d="M2 4C2 4 4 6 8 6C12 6 14 4 14 4"/><path d="M2 8C2 8 4 10 8 10C12 10 14 8 14 8"/></svg></div>
          <div class="stat-info">
            <div class="stat-value" :class="{ 'skeleton-inline': loadingSummary }">{{ loadingSummary ? '' : summary.buckets_count }}</div>
            <div class="stat-label">S3 Buckets</div>
          </div>
        </div>
        <div class="stat-card" @click="$router.push('/cloudformation')">
          <div class="stat-icon stat-icon--cf"><svg width="20" height="20" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"><path d="M2 4L8 1L14 4V12L8 15L2 12V4Z"/><path d="M8 7V15"/><path d="M2 4L8 7L14 4"/></svg></div>
          <div class="stat-info">
            <div class="stat-value" :class="{ 'skeleton-inline': loadingSummary }">{{ loadingSummary ? '' : summary.stacks_count }}</div>
            <div class="stat-label">CF Stacks</div>
          </div>
        </div>
        <div class="stat-card stat-card--success" @click="$router.push('/cloudformation')">
          <div class="stat-icon stat-icon--active"><svg width="20" height="20" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M3 8.5L6.5 12L13 4"/></svg></div>
          <div class="stat-info">
            <div class="stat-value" :class="{ 'skeleton-inline': loadingSummary }">{{ loadingSummary ? '' : summary.active_stacks }}</div>
            <div class="stat-label">Active Stacks</div>
          </div>
        </div>
        <div class="stat-card" :class="{ 'stat-card--danger': summary.failed_stacks > 0 }" @click="$router.push('/cloudformation')">
          <div class="stat-icon stat-icon--failed"><svg width="20" height="20" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M8 4V9"/><circle cx="8" cy="12" r="0.75" fill="currentColor" stroke="none"/><path d="M2 14L8 2L14 14H2Z" stroke-linejoin="round"/></svg></div>
          <div class="stat-info">
            <div class="stat-value" :class="{ 'skeleton-inline': loadingSummary }">{{ loadingSummary ? '' : summary.failed_stacks }}</div>
            <div class="stat-label">Failed Stacks</div>
          </div>
        </div>
        <div class="stat-card" @click="$router.push('/billing')">
          <div class="stat-icon stat-icon--billing"><svg width="20" height="20" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3"><rect x="2" y="3" width="12" height="10" rx="1.5"/><path d="M2 6H14"/><path d="M5 9H8" stroke-linecap="round"/><path d="M5 11H6.5" stroke-linecap="round"/></svg></div>
          <div class="stat-info">
            <div class="stat-value" :class="{ 'skeleton-inline': loadingCost }">{{ loadingCost ? '' : formatCurrency(monthlyCost) }}</div>
            <div class="stat-label">This Month</div>
          </div>
        </div>
      </div>

      <!-- Two columns: Stacks + Buckets -->
      <div class="dashboard-grid">
        <!-- Recent Stacks -->
        <div class="dash-panel">
          <div class="dash-panel-header">
            <h3>CloudFormation Stacks</h3>
            <router-link to="/cloudformation" class="dash-link">View all →</router-link>
          </div>
          <div v-if="loadingSummary" class="dash-panel-loading">
            <div v-for="i in 3" :key="i" class="skeleton" style="width:100%;height:36px;border-radius:6px;margin-bottom:6px"></div>
          </div>
          <div v-else-if="summary.stacks.length === 0" class="dash-panel-empty">
            <p class="text-tertiary text-sm">No stacks deployed</p>
          </div>
          <div v-else class="dash-stack-list">
            <div v-for="stack in summary.stacks.slice(0, 6)" :key="stack.name" class="dash-stack-row">
              <span class="dash-stack-name">{{ stack.name }}</span>
              <span :class="statusBadgeClass(stack.status)" class="badge badge-sm">{{ formatStatus(stack.status) }}</span>
            </div>
          </div>
        </div>

        <!-- Recent Buckets -->
        <div class="dash-panel">
          <div class="dash-panel-header">
            <h3>S3 Buckets</h3>
            <router-link to="/s3" class="dash-link">View all →</router-link>
          </div>
          <div v-if="loadingSummary" class="dash-panel-loading">
            <div v-for="i in 3" :key="i" class="skeleton" style="width:100%;height:36px;border-radius:6px;margin-bottom:6px"></div>
          </div>
          <div v-else-if="summary.buckets.length === 0" class="dash-panel-empty">
            <p class="text-tertiary text-sm">No buckets found</p>
          </div>
          <div v-else class="dash-bucket-list">
            <router-link v-for="bucket in summary.buckets.slice(0, 8)" :key="bucket.name" :to="`/s3/${bucket.name}`" class="dash-bucket-row">
              <span class="dash-bucket-icon"><svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3"><path d="M2 4C2 4 4 2 8 2C12 2 14 4 14 4V12C14 12 12 14 8 14C4 14 2 12 2 12V4Z"/><path d="M2 4C2 4 4 6 8 6C12 6 14 4 14 4"/><path d="M2 8C2 8 4 10 8 10C12 10 14 8 14 8"/></svg></span>
              <span class="dash-bucket-name">{{ bucket.name }}</span>
              <span class="text-tertiary text-sm">{{ formatDate(bucket.creation_date) }}</span>
            </router-link>
          </div>
        </div>
      </div>
    </template>

    <!-- Add Account Form -->
    <section v-if="showAddForm || accountsStore.accounts.length === 0" class="section">
      <div class="glass-panel form-panel">
        <div class="form-panel-header">
          <h2>Add AWS Account</h2>
          <button v-if="accountsStore.accounts.length > 0" class="btn-secondary" @click="showAddForm = false">Cancel</button>
        </div>
        <p class="text-secondary text-sm" style="margin-bottom: 1.5rem;">
          Your credentials are verified with AWS STS and stored locally.
        </p>
        <form @submit.prevent="addAccount" class="account-form">
          <div class="form-row">
            <div class="form-group">
              <label>Account Name</label>
              <input v-model="form.name" type="text" class="glass-input" placeholder="Production" required :disabled="loading" />
            </div>
            <div class="form-group">
              <label>Region</label>
              <input v-model="form.region" type="text" class="glass-input" placeholder="us-east-1" required :disabled="loading" />
            </div>
          </div>
          <div class="form-group">
            <label>Access Key ID</label>
            <input v-model="form.accessKeyId" type="text" class="glass-input" placeholder="AKIA..." required :disabled="loading" autocomplete="off" />
          </div>
          <div class="form-group">
            <label>Secret Access Key</label>
            <input v-model="form.secretAccessKey" type="password" class="glass-input" required :disabled="loading" autocomplete="off" />
          </div>
          <div v-if="error" class="error-banner">{{ error }}</div>
          <div class="form-actions">
            <button type="submit" class="btn-primary" :disabled="loading">
              <svg v-if="loading" width="14" height="14" viewBox="0 0 16 16" fill="none" class="spinning"><path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
              {{ loading ? 'Verifying...' : 'Connect Account' }}
            </button>
          </div>
        </form>
      </div>
    </section>

    <!-- Connected Accounts -->
    <section v-if="accountsStore.accounts.length > 0" class="section">
      <div class="section-header">
        <h2>Connected Accounts</h2>
        <button class="btn-secondary" @click="showAddForm = true" v-if="!showAddForm">+ Add</button>
      </div>
      <div class="accounts-grid">
        <div v-for="acc in accountsStore.accounts" :key="acc.id" class="account-card" :class="{ 'account-card--active': acc.id === accountsStore.activeAccountId }">
          <div class="account-card-header">
            <div class="account-avatar">{{ acc.name.charAt(0).toUpperCase() }}</div>
            <div class="account-details">
              <div class="account-name">{{ acc.name }}</div>
              <div class="account-meta"><span>{{ acc.region }}</span><span v-if="acc.accountId">· {{ acc.accountId }}</span></div>
            </div>
          </div>
          <div class="account-card-actions">
            <span v-if="acc.id === accountsStore.activeAccountId" class="badge badge--success">Active</span>
            <button @click="removeAccount(acc.id, acc.name)" class="btn-danger">Remove</button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch, inject } from 'vue';
import { useAwsAccountsStore } from '../store/awsAccounts';
import { useBillingStore } from '../store/billing';
import { invoke } from '@tauri-apps/api/core';

const accountsStore = useAwsAccountsStore();
const billingStore = useBillingStore();
const toast = inject<any>('toast');

const showAddForm = ref(false);
const form = ref({ name: '', accessKeyId: '', secretAccessKey: '', region: 'us-east-1' });
const loading = ref(false);
const error = ref('');

// Dashboard data
const loadingSummary = ref(false);
const loadingCost = ref(false);
const monthlyCost = ref<string | null>(null);

const summary = reactive({
  buckets_count: 0,
  buckets: [] as { name: string; creation_date: string | null }[],
  stacks_count: 0,
  stacks: [] as { name: string; status: string; creation_time: string | null }[],
  active_stacks: 0,
  failed_stacks: 0,
});

function getAccount() {
  const id = accountsStore.activeAccountId;
  return id ? accountsStore.accounts.find(a => a.id === id) || null : null;
}

async function fetchSummary() {
  const acc = getAccount(); if (!acc) return;
  loadingSummary.value = true;
  try {
    const res: any = await invoke('get_dashboard_summary', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region,
    });
    if (res.success) {
      summary.buckets_count = res.buckets_count;
      summary.buckets = res.buckets;
      summary.stacks_count = res.stacks_count;
      summary.stacks = res.stacks;
      summary.active_stacks = res.active_stacks;
      summary.failed_stacks = res.failed_stacks;
    }
  } catch (_) {}
  finally { loadingSummary.value = false; }
}

async function fetchCost() {
  const acc = getAccount(); if (!acc) return;
  const id = accountsStore.activeAccountId!;
  loadingCost.value = true;
  try {
    const cached = billingStore.getCacheForAccount(id);
    if (cached) {
      monthlyCost.value = cached.costSummary.currentMonthTotal;
    } else {
      const res = await billingStore.fetchCostSummary(id, acc.accessKeyId, acc.secretAccessKey, acc.region, false);
      if (res) monthlyCost.value = res.currentMonthTotal;
    }
  } catch (_) {}
  finally { loadingCost.value = false; }
}

function refreshAll() {
  fetchSummary();
  fetchCost();
}

function formatCurrency(value: string | null): string {
  if (!value) return '$—';
  const num = parseFloat(value);
  if (isNaN(num)) return '$0.00';
  return `$${num.toFixed(2)}`;
}

function formatDate(d: string | null): string {
  if (!d) return '';
  return new Date(d).toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
}

function formatStatus(status: string): string {
  return status.replace(/_/g, ' ').toLowerCase().replace(/\b\w/g, c => c.toUpperCase());
}

function statusBadgeClass(status: string): string {
  if (status.includes('COMPLETE') && !status.includes('DELETE')) return 'badge--success';
  if (status.includes('FAILED') || status.includes('ROLLBACK')) return 'badge--error';
  if (status.includes('IN_PROGRESS')) return 'badge--warning';
  return 'badge--neutral';
}

async function addAccount() {
  loading.value = true; error.value = '';
  try {
    await accountsStore.saveAccount({ name: form.value.name, accessKeyId: form.value.accessKeyId, secretAccessKey: form.value.secretAccessKey, region: form.value.region });
    toast?.success(`Account "${form.value.name}" connected`, 'Added');
    form.value = { name: '', accessKeyId: '', secretAccessKey: '', region: 'us-east-1' };
    showAddForm.value = false;
    refreshAll();
  } catch (err: any) { error.value = err.message || 'Verification failed'; }
  finally { loading.value = false; }
}

function removeAccount(id: string, name: string) {
  accountsStore.deleteAccount(id);
  toast?.info(`"${name}" removed`, 'Removed');
}

onMounted(() => {
  accountsStore.loadAccounts();
  if (accountsStore.activeAccountId) refreshAll();
});

watch(() => accountsStore.activeAccountId, (newId) => {
  if (newId) refreshAll();
});
</script>

<style scoped>
.dashboard-view { max-width: 1024px; }
.page-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 1.5rem; }
.page-header h1 { margin-bottom: 0.25rem; }
.page-actions { display: flex; gap: 0.5rem; }

/* Empty State */
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; padding: 4rem 2rem; gap: 1rem; }
.empty-icon { color: var(--text-tertiary); margin-bottom: 0.5rem; }
.empty-state h2 { margin-bottom: 0; }

/* Stat Cards */
.stat-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 0.625rem; margin-bottom: 1.5rem; }
.stat-card { display: flex; align-items: center; gap: 0.75rem; padding: 1.125rem 1rem; background: var(--bg-primary); border: 1px solid var(--border-default); border-radius: var(--radius-lg); cursor: pointer; transition: all var(--transition-fast); }
.stat-card:hover { border-color: var(--border-hover); background: var(--bg-secondary); transform: translateY(-1px); box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15); }
.stat-card--success { border-color: rgba(12, 206, 107, 0.2); background: rgba(12, 206, 107, 0.03); }
.stat-card--danger { border-color: rgba(238, 0, 0, 0.2); background: rgba(238, 0, 0, 0.03); }
.stat-icon { font-size: 1.375rem; width: 40px; height: 40px; display: flex; align-items: center; justify-content: center; border-radius: 10px; background: var(--bg-elevated); border: 1px solid var(--border-default); flex-shrink: 0; }
.stat-info { display: flex; flex-direction: column; gap: 0.125rem; }
.stat-value { font-size: 1.375rem; font-weight: 700; color: var(--text-primary); font-variant-numeric: tabular-nums; line-height: 1.2; }
.stat-label { font-size: 0.6875rem; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.04em; font-weight: 500; }
.skeleton-inline { background: var(--bg-elevated); border-radius: 4px; width: 40px; height: 1.375rem; animation: pulse 1.5s ease infinite; display: inline-block; }
@keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.4; } }

/* Dashboard Grid */
.dashboard-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; margin-bottom: 1.5rem; }
.dash-panel { background: var(--bg-primary); border: 1px solid var(--border-default); border-radius: var(--radius-lg); padding: 1.25rem; min-height: 220px; }
.dash-panel-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; padding-bottom: 0.75rem; border-bottom: 1px solid var(--border-default); }
.dash-panel-header h3 { margin: 0; font-size: 0.875rem; }
.dash-link { font-size: 0.75rem; color: var(--accent); text-decoration: none; transition: opacity var(--transition-fast); }
.dash-link:hover { opacity: 0.8; }
.dash-panel-loading { display: flex; flex-direction: column; gap: 0.5rem; }
.dash-panel-empty { padding: 2rem; text-align: center; }

/* Stack List */
.dash-stack-list { display: flex; flex-direction: column; gap: 0.25rem; }
.dash-stack-row { display: flex; align-items: center; justify-content: space-between; padding: 0.5rem 0.75rem; border-radius: var(--radius-sm); border: 1px solid transparent; transition: all var(--transition-fast); }
.dash-stack-row:hover { background: var(--bg-hover); border-color: var(--border-default); }
.dash-stack-name { font-size: 0.8125rem; font-weight: 500; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 60%; }
.badge-sm { font-size: 0.625rem; padding: 0.125rem 0.375rem; }

/* Bucket List */
.dash-bucket-list { display: flex; flex-direction: column; gap: 0.25rem; }
.dash-bucket-row { display: flex; align-items: center; gap: 0.625rem; padding: 0.5rem 0.75rem; border-radius: var(--radius-sm); text-decoration: none; border: 1px solid transparent; transition: all var(--transition-fast); }
.dash-bucket-row:hover { background: var(--bg-hover); border-color: var(--border-default); }
.dash-bucket-icon { font-size: 1rem; }
.dash-bucket-name { flex: 1; font-size: 0.8125rem; font-weight: 500; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

/* Sections */
.section { margin-bottom: 1.5rem; }
.section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.75rem; }
.section-header h2 { margin-bottom: 0; }

/* Form */
.form-panel { padding: 1.5rem; }
.form-panel-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem; }
.form-panel-header h2 { margin-bottom: 0; }
.account-form { display: flex; flex-direction: column; gap: 1rem; max-width: 520px; }
.form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
.form-group { display: flex; flex-direction: column; gap: 0.375rem; }
.form-group label { font-size: 0.75rem; font-weight: 500; color: var(--text-secondary); }
.form-actions { margin-top: 0.5rem; }
.error-banner { display: flex; align-items: center; gap: 0.5rem; padding: 0.625rem 0.875rem; background: var(--error-subtle); border: 1px solid rgba(238,0,0,0.15); border-radius: var(--radius-sm); color: #ff6166; font-size: 0.8125rem; }

/* Accounts */
.accounts-grid { display: flex; flex-direction: column; gap: 0.5rem; }
.account-card { display: flex; align-items: center; justify-content: space-between; padding: 0.875rem 1rem; background: var(--bg-primary); border: 1px solid var(--border-default); border-radius: var(--radius-md); transition: all var(--transition-fast); }
.account-card--active { border-color: rgba(12, 206, 107, 0.2); }
.account-card:hover { border-color: var(--border-hover); }
.account-card-header { display: flex; align-items: center; gap: 0.75rem; }
.account-avatar { width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; background: var(--bg-elevated); border: 1px solid var(--border-default); border-radius: 8px; font-weight: 600; font-size: 0.8125rem; color: var(--text-secondary); }
.account-details { display: flex; flex-direction: column; }
.account-name { font-weight: 500; font-size: 0.875rem; }
.account-meta { font-size: 0.75rem; color: var(--text-tertiary); display: flex; gap: 0.25rem; }
.account-card-actions { display: flex; align-items: center; gap: 0.75rem; }

.spinning { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
