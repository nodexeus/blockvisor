<script>
  import UserDetailsItem from './UserDetailsItem.svelte';
</script>

<article class="user-details">
  <ul class="u-list-reset">
    <UserDetailsItem>
      <svelte:fragment slot="label">First Name</svelte:fragment>
      Mark
    </UserDetailsItem>
    <UserDetailsItem>
      <svelte:fragment slot="label">Last Name</svelte:fragment>
      Bryant
    </UserDetailsItem>
    <UserDetailsItem>
      <svelte:fragment slot="label">E-Mail</svelte:fragment>
      mbriant@company.com
    </UserDetailsItem>
    <UserDetailsItem>
      <svelte:fragment slot="label">Organization</svelte:fragment>
      Grounded Skynet LTD.
    </UserDetailsItem>
    <UserDetailsItem>
      <svelte:fragment slot="label">Groups</svelte:fragment>
      Grounded Skynet LTD.
    </UserDetailsItem>
  </ul>
</article>

<style>
  .user-details {
    margin-top: 40px;
  }
</style>
