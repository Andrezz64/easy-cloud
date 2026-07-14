<template>
  <div class="app-container">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="logo-area">
        <div class="logo-mark">
          <img src="/logo.svg" alt="Easy Cloud Logo" style="width: 100%; height: 100%; object-fit: contain;" />
        </div>
        <span class="logo-text">Easy Cloud</span>
      </div>

      <nav class="nav-menu">
        <router-link to="/" class="nav-link" active-class="active" exact>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <rect x="1" y="1" width="6" height="6" rx="1.5" stroke="currentColor" stroke-width="1.3"/>
            <rect x="9" y="1" width="6" height="6" rx="1.5" stroke="currentColor" stroke-width="1.3"/>
            <rect x="1" y="9" width="6" height="6" rx="1.5" stroke="currentColor" stroke-width="1.3"/>
            <rect x="9" y="9" width="6" height="6" rx="1.5" stroke="currentColor" stroke-width="1.3"/>
          </svg>
          <span>Dashboard</span>
        </router-link>
        <router-link to="/s3" class="nav-link" active-class="active">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M2 4C2 4 4 2 8 2C12 2 14 4 14 4V12C14 12 12 14 8 14C4 14 2 12 2 12V4Z" stroke="currentColor" stroke-width="1.3"/>
            <path d="M2 4C2 4 4 6 8 6C12 6 14 4 14 4" stroke="currentColor" stroke-width="1.3"/>
            <path d="M2 8C2 8 4 10 8 10C12 10 14 8 14 8" stroke="currentColor" stroke-width="1.3"/>
          </svg>
          <span>S3 Storage</span>
        </router-link>
        <router-link to="/cloudformation" class="nav-link" active-class="active">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M2 4L8 1L14 4V12L8 15L2 12V4Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/>
            <path d="M8 7V15" stroke="currentColor" stroke-width="1.3"/>
            <path d="M2 4L8 7L14 4" stroke="currentColor" stroke-width="1.3"/>
          </svg>
          <span>CloudFormation</span>
        </router-link>
        <router-link to="/billing" class="nav-link" active-class="active">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <rect x="2" y="3" width="12" height="10" rx="1.5" stroke="currentColor" stroke-width="1.3"/>
            <path d="M2 6H14" stroke="currentColor" stroke-width="1.3"/>
            <path d="M5 9H8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <path d="M5 11H6.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          </svg>
          <span>Billing</span>
        </router-link>
      </nav>

      <div class="sidebar-footer">
      </div>
    </aside>

    <!-- Main -->
    <main class="main-content">
      <header class="topbar">
        <div class="topbar-left">
          <div class="breadcrumb">
            <span class="breadcrumb-root">Easy Cloud</span>
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" class="breadcrumb-sep">
              <path d="M6 4L10 8L6 12" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            </svg>
            <span class="breadcrumb-current">{{ currentPageName }}</span>
          </div>
          <button 
            v-if="accountsStore.activeAccountId" 
            class="btn-icon"
            @click="syncData"
            :disabled="syncing"
            title="Sync AWS Data"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" :class="{ spinning: syncing }">
              <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
              <path d="M14 2V6H10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </button>
        </div>
        <div class="topbar-right">
          <div class="topbar-account" v-if="accountsStore.accounts.length > 0">
            <span class="status-dot" v-if="accountsStore.activeAccountId"></span>
            <select 
              class="glass-select topbar-select" 
              v-model="accountsStore.activeAccountId" 
              @change="onAccountChange"
            >
              <option value="" disabled>Select account</option>
              <option 
                v-for="acc in accountsStore.accounts" 
                :key="acc.id" 
                :value="acc.id"
              >
                {{ acc.name }}
              </option>
            </select>
          </div>
        </div>
      </header>

      <div class="content-area">
        <router-view v-slot="{ Component }">
          <Transition name="slide" mode="out-in">
            <component :is="Component" ref="currentView" />
          </Transition>
        </router-view>
      </div>
    </main>

    <!-- Toast -->
    <ToastNotification ref="toastRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, provide, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useAwsAccountsStore } from './store/awsAccounts';
import ToastNotification from './components/ToastNotification.vue';

const route = useRoute();
const accountsStore = useAwsAccountsStore();
const toastRef = ref<InstanceType<typeof ToastNotification> | null>(null);
const currentView = ref(null);
const syncing = ref(false);

const currentPageName = computed(() => {
  const name = route.name as string;
  if (!name) return '';
  return name.charAt(0).toUpperCase() + name.slice(1);
});

// Provide toast globally
provide('toast', {
  success: (msg: string, title?: string) => toastRef.value?.addToast('success', msg, title),
  error: (msg: string, title?: string) => toastRef.value?.addToast('error', msg, title),
  warning: (msg: string, title?: string) => toastRef.value?.addToast('warning', msg, title),
  info: (msg: string, title?: string) => toastRef.value?.addToast('info', msg, title),
});

onMounted(() => {
  accountsStore.loadAccounts();
});

const onAccountChange = () => {
  toastRef.value?.addToast('info', `Switched to account`, 'Account Changed');
};

const syncData = async () => {
  syncing.value = true;
  // Simulated sync — in real app this would refetch all data
  await new Promise(resolve => setTimeout(resolve, 1500));
  syncing.value = false;
  toastRef.value?.addToast('success', 'All resources synced from AWS', 'Sync Complete');
};
</script>

<style scoped>
/* Logo */
.logo-area {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0 0.5rem;
  margin-bottom: 2rem;
}

.logo-mark {
  width: 50px;
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-primary);
}

.logo-text {
  font-weight: 600;
  font-size: 1.1rem;
  letter-spacing: -0.02em;
  color: var(--text-primary);
}

/* Navigation */
.nav-menu {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-link {
  text-decoration: none;
  color: var(--text-secondary);
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  font-weight: 450;
  font-size: 0.8125rem;
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.nav-link svg {
  opacity: 0.7;
  transition: opacity var(--transition-fast);
}

.nav-link:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.nav-link:hover svg {
  opacity: 1;
}

.nav-link.active {
  color: var(--text-primary);
  background: var(--bg-elevated);
}

.nav-link.active svg {
  opacity: 1;
}

/* Sidebar Footer */
.sidebar-footer {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--success-subtle);
  border: 1px solid rgba(12, 206, 107, 0.15);
  border-radius: var(--radius-sm);
}

.status-dot {
  width: 6px;
  height: 6px;
  background: var(--success);
  border-radius: 50%;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.status-text {
  font-size: 0.6875rem;
  color: var(--success);
  font-weight: 500;
}

/* Topbar */
.topbar-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.breadcrumb-root {
  color: var(--text-tertiary);
  font-size: 0.8125rem;
}

.breadcrumb-sep {
  color: var(--text-tertiary);
}

.breadcrumb-current {
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-weight: 500;
}

.btn-icon {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-icon:hover:not(:disabled) {
  color: var(--text-primary);
  border-color: var(--border-hover);
  background: var(--bg-hover);
}

.btn-icon:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.topbar-account {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.topbar-account .status-dot {
  width: 6px;
  height: 6px;
  background: var(--success);
  border-radius: 50%;
  animation: pulse 2s ease-in-out infinite;
  flex-shrink: 0;
}

.topbar-select {
  width: auto;
  min-width: 140px;
  padding: 0.375rem 2rem 0.375rem 0.625rem;
  font-size: 0.75rem;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Page transitions */
.slide-enter-active,
.slide-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.slide-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
