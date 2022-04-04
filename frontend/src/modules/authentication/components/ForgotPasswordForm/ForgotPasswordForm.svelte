<script lang="ts">
  import type { UiContainer } from '@ory/kratos-client';
  import { useForm, required, email } from 'svelte-use-form';
  import FormBuilder from '../FormBuilder/FormBuilder.svelte';

  export let ui: UiContainer;

  const validationMapper = {
    email: [required, email],
  };

  const form = useForm();
</script>

<form
  enctype="application/x-www-form-urlencoded"
  method={ui.method}
  action={ui.action}
  class="forgot-password"
  use:form
>
  <ul class="u-list-reset forgot-password__input-list">
    <FormBuilder {validationMapper} {form} nodes={ui.nodes} />
  </ul>
</form>

<style>
  .forgot-password__input-list {
    margin-bottom: 24px;
  }
</style>
