import { createRouter, createWebHistory } from 'vue-router';
import DashboardView from '../views/DashboardView.vue';
import CloudFormationView from '../views/CloudFormationView.vue';
import S3View from '../views/S3View.vue';
import S3BucketView from '../views/S3BucketView.vue';
import BillingView from '../views/BillingView.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: DashboardView
    },
    {
      path: '/s3',
      name: 's3',
      component: S3View
    },
    {
      path: '/s3/:bucket',
      name: 's3-bucket',
      component: S3BucketView
    },
    {
      path: '/cloudformation',
      name: 'cloudformation',
      component: CloudFormationView
    },
    {
      path: '/billing',
      name: 'billing',
      component: BillingView
    }
  ]
});

export default router;
