<script lang="ts">
  import Toggle from 'modules/forms/components/Toggle/Toggle.svelte';
  import { useForm } from 'svelte-use-form';

  const form = useForm();
</script>

<article class="notifications">
  <div class="notifications__title t-medium">E-mail Notifications</div>
  <form class="notifications__form" use:form>
    <ul class="u-list-reset">
      <li class="notifications__item">
        <Toggle name="emailNotifications" field={$form.emailNotifications}>
          E-mail Notifications
          <svelte:fragment slot="description"
            >Get emails to find out what's going on</svelte:fragment
          >
        </Toggle>
      </li>
      <li class="notifications__item">
        <Toggle
          name="earningsNotifications"
          field={$form.earningsNotifications}
        >
          E-mail Notifications
          <svelte:fragment slot="description"
            >Get emails to find out what's going on</svelte:fragment
          >
        </Toggle>
      </li>
      <li class="notifications__item">
        <Toggle name="stateNotifications" field={$form.stateNotifications}>
          E-mail Notifications
          <svelte:fragment slot="description"
            >Get emails to find out what's going on</svelte:fragment
          >
        </Toggle>
      </li>
    </ul>
  </form>
</article>

<style>
  .notifications {
    margin-top: 60px;
    margin-bottom: 40px;
    display: flex;
    flex-direction: column;
    gap: 32px 28px;

    &__form {
      flex-grow: 1;
    }

    &__item {
      & + :global(.notifications__item) {
        margin-top: 20px;
        padding-top: 20px;
        border-top: 1px solid theme(--color-text-5-o10);
      }
    }

    @media (--screen-medium-small) {
      flex-direction: row;
    }

    &__title {
      @media (--screen-medium-small) {
        flex: 0 0 188px;
      }
    }
  }
</style>
