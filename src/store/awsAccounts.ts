import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface AwsAccount {
  id: string;
  name: string;
  accessKeyId: string;
  secretAccessKey: string;
  region: string;
  arn?: string;
  accountId?: string;
}

export interface VerifyResponse {
  success: boolean;
  account?: string;
  arn?: string;
  error_message?: string;
}

export const useAwsAccountsStore = defineStore('awsAccounts', () => {
  const accounts = ref<AwsAccount[]>([]);
  const activeAccountId = ref<string | null>(null);

  async function loadAccounts() {
    const stored = localStorage.getItem('aws_accounts');
    if (stored) {
      accounts.value = JSON.parse(stored);
      if (accounts.value.length > 0 && !activeAccountId.value) {
        activeAccountId.value = accounts.value[0].id;
      }
    }
  }

  async function saveAccount(account: Omit<AwsAccount, 'id' | 'arn' | 'accountId'>) {
    // Verificar credenciais na AWS via Rust Backend
    try {
      const response: VerifyResponse = await invoke('verify_aws_credentials', {
        accessKeyId: account.accessKeyId,
        secretAccessKey: account.secretAccessKey,
        region: account.region
      });

      if (!response.success) {
        throw new Error(response.error_message || "Invalid credentials");
      }

      const newAccount: AwsAccount = {
        ...account,
        id: crypto.randomUUID(),
        arn: response.arn,
        accountId: response.account
      };
      
      accounts.value.push(newAccount);
      localStorage.setItem('aws_accounts', JSON.stringify(accounts.value));
      
      if (!activeAccountId.value) {
        activeAccountId.value = newAccount.id;
      }
      
      return true;
    } catch (err) {
      console.error("Erro ao verificar credenciais: ", err);
      throw err;
    }
  }

  function setActiveAccount(id: string) {
    activeAccountId.value = id;
  }

  function deleteAccount(id: string) {
    accounts.value = accounts.value.filter(a => a.id !== id);
    localStorage.setItem('aws_accounts', JSON.stringify(accounts.value));
    if (activeAccountId.value === id) {
      activeAccountId.value = accounts.value.length > 0 ? accounts.value[0].id : null;
    }
  }

  return {
    accounts,
    activeAccountId,
    loadAccounts,
    saveAccount,
    setActiveAccount,
    deleteAccount
  };
});
