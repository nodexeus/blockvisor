<script>
  import { ROUTES } from 'consts/routes';

  import InvoiceDetails from 'modules/admin-console/components/InvoiceDetails/InvoiceDetails.svelte';

  import InvoiceHeader from 'modules/admin-console/components/InvoiceHeader/InvoiceHeader.svelte';
  import { app } from 'modules/app/store';
  import { onMount } from 'svelte';

  onMount(() => {
    setTimeout(() => {
      app.setBreadcrumbs([
        {
          title: 'Admin Console',
          url: ROUTES.ADMIN_CONSOLE_DASHBOARD,
        },
        {
          title: 'User Management',
          url: ROUTES.ADMIN_CONSOLE_USER_MANAGEMENT,
        },
        {
          title: 'Invoices',
          url: `${ROUTES.ADMIN_CONSOLE_USER_EDIT('user-id')}?tab=3`,
        },
        {
          title: 'Invoice Preview',
          url: '',
        },
      ]);
    }, 200);
  });
</script>

<InvoiceHeader />

<InvoiceDetails />
