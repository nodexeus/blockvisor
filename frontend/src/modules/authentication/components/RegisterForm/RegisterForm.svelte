<script lang="ts">
  import type { UiContainer } from '@ory/kratos-client';

  import { reorderFields } from 'modules/authentication/util/reorder-fields';

  import { useForm, required, email } from 'svelte-use-form';
  import { passwordMatchConfirm, passwordMatchPassword } from 'utils';
  import FormBuilder from '../FormBuilder/FormBuilder.svelte';

  export let ui: UiContainer;

  export let order = [
    'csrf_token',
    'traits.name.first',
    'traits.name.last',
    'traits.organization',
    'traits.email',
    'password',
  ];

  const form = useForm();

  const validationMapper = {
    confirmPassword: [required, passwordMatchPassword],
    password: [required, passwordMatchConfirm],
    'traits.email': [required, email],
    'traits.name.first': [required],
    'traits.name.last': [required],
  };
</script>

<form
  enctype="application/x-www-form-urlencoded"
  class="register"
  action={ui.action}
  method={ui.method}
  use:form
>
  <ul class="u-list-reset register__input-list">
    <FormBuilder
      hasPasswordConfirm
      {form}
      {validationMapper}
      nodes={reorderFields(ui.nodes, order)}
    />
  </ul>
</form>

<style>
  .register__input-list {
    margin-bottom: 24px;
  }
</style>
