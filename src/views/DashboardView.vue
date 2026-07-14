<template>
  <div class="dashboard-view">
    <!-- Header -->
    <div class="page-header">
      <div>
        <h1>Dashboard</h1>
        <p class="text-secondary text-sm">Overview of your AWS resources</p>
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
      <p class="text-secondary">Connect your AWS account to get started managing your cloud resources.</p>
      <button class="btn-primary" @click="showAddForm = true">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M8 3V13M3 8H13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        Add AWS Account
      </button>
    </div>

    <!-- S3 Buckets section -->
    <section v-if="accountsStore.activeAccountId" class="section">
      <div class="section-header">
        <div class="section-title-group">
          <h2>S3 Buckets</h2>
          <span class="badge badge--neutral" v-if="!loadingBuckets && buckets.length > 0">
            {{ buckets.length }}
          </span>
        </div>
        <button class="btn-secondary" @click="fetchBuckets" :disabled="loadingBuckets">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" :class="{ spinning: loadingBuckets }">
            <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <path d="M14 2V6H10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Refresh
        </button>
      </div>

      <!-- Error -->
      <div v-if="bucketsError" class="error-banner">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.3"/>
          <path d="M8 5V9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          <circle cx="8" cy="11.5" r="0.75" fill="currentColor"/>
        </svg>
        <span>{{ bucketsError }}</span>
      </div>

      <!-- Loading skeleton -->
      <div v-else-if="loadingBuckets" class="resource-grid">
        <div v-for="i in 6" :key="i" class="resource-card resource-card--skeleton">
          <div class="skeleton" style="width: 40px; height: 40px; border-radius: 8px;"></div>
          <div style="flex: 1; display: flex; flex-direction: column; gap: 6px;">
            <div class="skeleton" style="width: 70%; height: 14px;"></div>
            <div class="skeleton" style="width: 40%; height: 12px;"></div>
          </div>
        </div>
      </div>

      <!-- Empty buckets -->
      <div v-else-if="buckets.length === 0" class="empty-section">
        <p class="text-secondary text-sm">No S3 buckets found in this account and region.</p>
      </div>

      <!-- Bucket grid -->
      <div v-else class="resource-grid">
        <div v-for="bucket in buckets" :key="bucket.name" class="resource-card">
          <div class="resource-icon">
            <svg width="18" height="18" viewBox="0 0 16 16" fill="none">
              <path d="M2 4C2 4 4 2 8 2C12 2 14 4 14 4V12C14 12 12 14 8 14C4 14 2 12 2 12V4Z" stroke="currentColor" stroke-width="1.2"/>
              <path d="M2 4C2 4 4 6 8 6C12 6 14 4 14 4" stroke="currentColor" stroke-width="1.2"/>
              <path d="M2 8C2 8 4 10 8 10C12 10 14 8 14 8" stroke="currentColor" stroke-width="1.2"/>
            </svg>
          </div>
          <div class="resource-info">
            <div class="resource-name">{{ bucket.name }}</div>
            <div class="resource-meta">{{ formatDate(bucket.creation_date) }}</div>
          </div>
        </div>
      </div>
    </section>

    <!-- Add Account Section -->
    <section v-if="showAddForm || accountsStore.accounts.length === 0" class="section">
      <div class="glass-panel form-panel">
        <div class="form-panel-header">
          <h2>Add AWS Account</h2>
          <button 
            v-if="accountsStore.accounts.length > 0" 
            class="btn-secondary" 
            @click="showAddForm = false"
          >Cancel</button>
        </div>
        <p class="text-secondary text-sm" style="margin-bottom: 1.5rem;">
          Your credentials are verified with AWS STS and stored locally.
        </p>
        
        <form @submit.prevent="addAccount" class="account-form">
          <div class="form-row">
            <div class="form-group">
              <label>Account Name</label>
              <input 
                v-model="form.name" 
                type="text" 
                class="glass-input" 
                placeholder="Production" 
                required 
                :disabled="loading" 
              />
            </div>
            <div class="form-group">
              <label>Region</label>
              <input 
                v-model="form.region" 
                type="text" 
                class="glass-input" 
                placeholder="us-east-1" 
                required 
                :disabled="loading" 
              />
            </div>
          </div>
          <div class="form-group">
            <label>Access Key ID</label>
            <input 
              v-model="form.accessKeyId" 
              type="text" 
              class="glass-input" 
              placeholder="AKIA..." 
              required 
              :disabled="loading" 
              autocomplete="off"
            />
          </div>
          <div class="form-group">
            <label>Secret Access Key</label>
            <input 
              v-model="form.secretAccessKey" 
              type="password" 
              class="glass-input" 
              required 
              :disabled="loading"
              autocomplete="off" 
            />
          </div>

          <div v-if="error" class="error-banner">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.3"/>
              <path d="M8 5V9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
              <circle cx="8" cy="11.5" r="0.75" fill="currentColor"/>
            </svg>
            <span>{{ error }}</span>
          </div>

          <div class="form-actions">
            <button type="submit" class="btn-primary" :disabled="loading">
              <svg v-if="loading" width="14" height="14" viewBox="0 0 16 16" fill="none" class="spinning">
                <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
              {{ loading ? 'Verifying credentials...' : 'Connect Account' }}
            </button>
          </div>
        </form>
      </div>
    </section>

    <!-- Configured Accounts -->
    <section v-if="accountsStore.accounts.length > 0" class="section">
      <div class="section-header">
        <h2>Connected Accounts</h2>
        <button class="btn-secondary" @click="showAddForm = true" v-if="!showAddForm">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M8 3V13M3 8H13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          Add
        </button>
      </div>
      
      <div class="accounts-grid">
        <div 
          v-for="acc in accountsStore.accounts" 
          :key="acc.id" 
          class="account-card"
          :class="{ 'account-card--active': acc.id === accountsStore.activeAccountId }"
        >
          <div class="account-card-header">
            <div class="account-avatar">{{ acc.name.charAt(0).toUpperCase() }}</div>
            <div class="account-details">
              <div class="account-name">{{ acc.name }}</div>
              <div class="account-meta">
                <span>{{ acc.region }}</span>
                <span v-if="acc.accountId">· {{ acc.accountId }}</span>
              </div>
            </div>
          </div>
          <div class="account-card-actions">
            <span 
              v-if="acc.id === accountsStore.activeAccountId" 
              class="badge badge--success"
            >Active</span>
            <button 
              @click="removeAccount(acc.id, acc.name)" 
              class="btn-danger"
            >Remove</button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, inject } from 'vue';
import { useAwsAccountsStore } from '../store/awsAccounts';
import { invoke } from '@tauri-apps/api/core';

const accountsStore = useAwsAccountsStore();
const toast = inject<any>('toast');

const showAddForm = ref(false);

const form = ref({
  name: '',
  accessKeyId: '',
  secretAccessKey: '',
  region: 'us-east-1'
});

const loading = ref(false);
const error = ref('');

// AWS Resources State
interface S3Bucket {
  name: string;
  creation_date: string | null;
}
const buckets = ref<S3Bucket[]>([]);
const loadingBuckets = ref(false);
const bucketsError = ref('');

onMounted(() => {
  accountsStore.loadAccounts();
  if (accountsStore.activeAccountId) {
    fetchBuckets();
  }
});

watch(() => accountsStore.activeAccountId, (newId) => {
  if (newId) {
    fetchBuckets();
  } else {
    buckets.value = [];
  }
});

const formatDate = (dateStr: string | null) => {
  if (!dateStr) return 'Unknown date';
  const d = new Date(dateStr);
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
};

const fetchBuckets = async () => {
  const activeId = accountsStore.activeAccountId;
  if (!activeId) return;
  const acc = accountsStore.accounts.find(a => a.id === activeId);
  if (!acc) return;

  loadingBuckets.value = true;
  bucketsError.value = '';
  
  try {
    const res: any = await invoke('list_s3_buckets', {
      accessKeyId: acc.accessKeyId,
      secretAccessKey: acc.secretAccessKey,
      region: acc.region
    });
    
    if (res.success) {
      buckets.value = res.buckets || [];
    } else {
      bucketsError.value = res.error_message || 'Failed to fetch buckets from AWS.';
    }
  } catch (err: any) {
    bucketsError.value = err.message || err.toString();
  } finally {
    loadingBuckets.value = false;
  }
};

const addAccount = async () => {
  loading.value = true;
  error.value = '';
  try {
    await accountsStore.saveAccount({
      name: form.value.name,
      accessKeyId: form.value.accessKeyId,
      secretAccessKey: form.value.secretAccessKey,
      region: form.value.region
    });
    
    toast?.success(`Account "${form.value.name}" connected successfully`, 'Account Added');
    
    // Reset form
    form.value.name = '';
    form.value.accessKeyId = '';
    form.value.secretAccessKey = '';
    showAddForm.value = false;
    
    fetchBuckets();
  } catch (err: any) {
    error.value = err.message || 'An error occurred while verifying the AWS credentials.';
    toast?.error(error.value, 'Connection Failed');
  } finally {
    loading.value = false;
  }
};

const removeAccount = (id: string, name: string) => {
  accountsStore.deleteAccount(id);
  toast?.info(`Account "${name}" removed`, 'Account Removed');
};
</script>

<style scoped>
.dashboard-view {
  max-width: 960px;
}

/* Page Header */
.page-header {
  margin-bottom: 2rem;
}

.page-header h1 {
  margin-bottom: 0.25rem;
}

/* Sections */
.section {
  margin-bottom: 2rem;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.section-title-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.section-header h2 {
  margin-bottom: 0;
}

/* Empty State */
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
  margin-bottom: 0.5rem;
}

.empty-state h2 {
  margin-bottom: 0;
}

.empty-state p {
  max-width: 320px;
}

.empty-section {
  padding: 2rem;
  text-align: center;
  border: 1px dashed var(--border-default);
  border-radius: var(--radius-md);
}

/* Resource Grid */
.resource-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 0.5rem;
}

.resource-card {
  display: flex;
  align-items: center;
  gap: 0.875rem;
  padding: 0.875rem 1rem;
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
  cursor: default;
}

.resource-card:hover {
  border-color: var(--border-hover);
  background: var(--bg-secondary);
}

.resource-card--skeleton {
  pointer-events: none;
}

.resource-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.resource-info {
  min-width: 0;
  flex: 1;
}

.resource-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.resource-meta {
  font-size: 0.75rem;
  color: var(--text-tertiary);
  margin-top: 2px;
}

/* Form Panel */
.form-panel {
  padding: 1.5rem;
}

.form-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.form-panel-header h2 {
  margin-bottom: 0;
}

.account-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-width: 520px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.form-group label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.form-actions {
  margin-top: 0.5rem;
}

/* Error Banner */
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

/* Accounts Grid */
.accounts-grid {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.account-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.875rem 1rem;
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.account-card--active {
  border-color: rgba(12, 206, 107, 0.2);
}

.account-card:hover {
  border-color: var(--border-hover);
}

.account-card-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.account-avatar {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  font-weight: 600;
  font-size: 0.8125rem;
  color: var(--text-secondary);
}

.account-details {
  display: flex;
  flex-direction: column;
}

.account-name {
  font-weight: 500;
  font-size: 0.875rem;
}

.account-meta {
  font-size: 0.75rem;
  color: var(--text-tertiary);
  display: flex;
  gap: 0.25rem;
}

.account-card-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

/* Utility */
.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
