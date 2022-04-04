<script lang="ts">
  import Input from 'modules/forms/components/Input/Input.svelte';
  import IconSearch from 'icons/search-12.svg';

  import { useForm } from 'svelte-use-form';

  const form = useForm();
</script>

<article class="controls">
  <form
    class="controls__search"
    use:form
    on:change={() => {
      console.log($form);
    }}
  >
    <Input
      size="small"
      style="outline"
      placeholder="Search by name or e-mail"
      name="search"
      field={$form?.search}
      labelClass="visually-hidden"
    >
      <IconSearch slot="utilLeft" />
      <svelte:fragment slot="label">Search by name or e-mail</svelte:fragment>
    </Input>
  </form>

  <slot name="action" />
</article>

<style>
  .controls {
    margin-top: 20px;
    margin-bottom: 60px;
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    gap: 16px 120px;

    &__search {
      flex-basis: 296px;
    }
  }
</style>
