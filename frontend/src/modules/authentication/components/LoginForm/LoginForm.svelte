<script lang="ts">
  import type { UiContainer } from '@ory/kratos-client';

  import { useForm, required, email } from 'svelte-use-form';
  import FormBuilder from '../FormBuilder/FormBuilder.svelte';

  const validationMapper = {
    password_identifier: [required],
    'traits.email': [required, email],
    password: [required],
  };

  export let ui: UiContainer;

  const form = useForm();
</script>

<form
  enctype="application/x-www-form-urlencoded"
  method={ui.method}
  action={ui.action}
  class="login"
  use:form
>
  <ul class="u-list-reset login__input-list">
    <FormBuilder {form} {validationMapper} nodes={ui.nodes} />
  </ul>
</form>

<style>
  .login__input-list {
    margin-bottom: 12px;
  }
</style>
