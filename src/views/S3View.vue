<template>
  <div class="s3-view">
    <!-- Header -->
    <div class="page-header">
      <div>
        <h1>S3 Storage</h1>
        <p class="text-secondary text-sm">Manage your S3 buckets and objects</p>
      </div>
      <div class="page-actions">
        <button 
          class="btn-secondary" 
          @click="fetchBuckets" 
          :disabled="loading || !accountsStore.activeAccountId"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" :class="{ spinning: loading }">
            <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <path d="M14 2V6H10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Refresh
        </button>
      </div>
    </div>

    <!-- No Account -->
    <div v-if="!accountsStore.activeAccountId" class="empty-state">
      <div class="empty-icon">
        <svg width="40" height="40" viewBox="0 0 16 16" fill="none">
          <path d="M2 4C2 4 4 2 8 2C12 2 14 4 14 4V12C14 12 12 14 8 14C4 14 2 12 2 12V4Z" stroke="currentColor" stroke-width="1"/>
          <path d="M2 4C2 4 4 6 8 6C12 6 14 4 14 4" stroke="currentColor" stroke-width="1"/>
          <path d="M2 8C2 8 4 10 8 10C12 10 14 8 14 8" stroke="currentColor" stroke-width="1"/>
        </svg>
      </div>
      <p class="text-secondary">Select an AWS account from the sidebar to browse S3 buckets.</p>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="error-banner">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.3"/>
        <path d="M8 5V9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
        <circle cx="8" cy="11.5" r="0.75" fill="currentColor"/>
      </svg>
      <span>{{ error }}</span>
    </div>

    <!-- Loading -->
    <div v-else-if="loading" class="buckets-grid">
      <div v-for="i in 8" :key="i" class="bucket-card bucket-card--skeleton">
        <div class="skeleton" style="width: 40px; height: 40px; border-radius: 10px;"></div>
        <div style="flex: 1; display: flex; flex-direction: column; gap: 8px;">
          <div class="skeleton" style="width: 65%; height: 14px;"></div>
          <div class="skeleton" style="width: 40%; height: 12px;"></div>
        </div>
      </div>
    </div>

    <!-- Empty -->
    <div v-else-if="buckets.length === 0" class="empty-state">
      <div class="empty-icon">
        <svg width="40" height="40" viewBox="0 0 16 16" fill="none">
          <path d="M2 4C2 4 4 2 8 2C12 2 14 4 14 4V12C14 12 12 14 8 14C4 14 2 12 2 12V4Z" stroke="currentColor" stroke-width="1"/>
          <path d="M2 4C2 4 4 6 8 6C12 6 14 4 14 4" stroke="currentColor" stroke-width="1"/>
        </svg>
      </div>
      <p class="text-secondary">No S3 buckets found in this account.</p>
    </div>

    <!-- Buckets Grid -->
    <div v-else class="buckets-grid">
      <router-link 
        v-for="bucket in buckets" 
        :key="bucket.name" 
        :to="`/s3/${bucket.name}`" 
        class="bucket-card"
      >
        <div class="bucket-icon">
          <svg width="20" height="20" viewBox="0 0 16 16" fill="none">
            <path d="M2 4C2 4 4 2 8 2C12 2 14 4 14 4V12C14 12 12 14 8 14C4 14 2 12 2 12V4Z" stroke="currentColor" stroke-width="1.2"/>
            <path d="M2 4C2 4 4 6 8 6C12 6 14 4 14 4" stroke="currentColor" stroke-width="1.2"/>
            <path d="M2 8C2 8 4 10 8 10C12 10 14 8 14 8" stroke="currentColor" stroke-width="1.2"/>
          </svg>
        </div>
        <div class="bucket-info">
          <div class="bucket-name">{{ bucket.name }}</div>
          <div class="bucket-meta">
            <span>Created {{ formatDate(bucket.creation_date) }}</span>
          </div>
        </div>
        <div class="bucket-arrow">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M6 4L10 8L6 12" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          </svg>
        </div>
      </router-link>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useAwsAccountsStore } from '../store/awsAccounts';
import { invoke } from '@tauri-apps/api/core';

const accountsStore = useAwsAccountsStore();

interface Bucket {
  name: string;
  creation_date: string | null;
}

const buckets = ref<Bucket[]>([]);
const loading = ref(false);
const error = ref('');

onMounted(() => {
  if (accountsStore.activeAccountId) {
    fetchBuckets();
  }
});

watch(() => accountsStore.activeAccountId, (newId) => {
  if (newId) fetchBuckets();
  else buckets.value = [];
});

const fetchBuckets = async () => {
  const activeId = accountsStore.activeAccountId;
  if (!activeId) return;
  const acc = accountsStore.accounts.find(a => a.id === activeId);
  if (!acc) return;

  loading.value = true;
  error.value = '';

  try {
    const res: any = await invoke('list_s3_buckets', {
      accessKeyId: acc.accessKeyId,
      secretAccessKey: acc.secretAccessKey,
      region: acc.region
    });

    if (res.success) {
      buckets.value = res.buckets || [];
    } else {
      error.value = res.error_message || 'Failed to fetch buckets.';
    }
  } catch (err: any) {
    error.value = err.message || err.toString();
  } finally {
    loading.value = false;
  }
};

const formatDate = (dateStr: string | null) => {
  if (!dateStr) return 'Unknown';
  const d = new Date(dateStr);
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
};
</script>

<style scoped>
.s3-view {
  max-width: 960px;
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
}

/* Buckets Grid */
.buckets-grid {
  display: flex;
  flex-direction: column;
  gap: 1px;
  background: var(--border-default);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.bucket-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.25rem;
  background: var(--bg-primary);
  text-decoration: none;
  color: inherit;
  transition: all var(--transition-fast);
  cursor: pointer;
}

.bucket-card:hover {
  background: var(--bg-secondary);
}

.bucket-card--skeleton {
  pointer-events: none;
}

.bucket-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 10px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.bucket-info {
  flex: 1;
  min-width: 0;
}

.bucket-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bucket-meta {
  font-size: 0.75rem;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.bucket-arrow {
  color: var(--text-tertiary);
  transition: all var(--transition-fast);
}

.bucket-card:hover .bucket-arrow {
  color: var(--text-secondary);
  transform: translateX(2px);
}

/* Error */
.error-banner {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: var(--error-subtle);
  border: 1px solid rgba(238, 0, 0, 0.15);
  border-radius: var(--radius-md);
  color: #ff6166;
  font-size: 0.8125rem;
}

/* Utilities */
.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
