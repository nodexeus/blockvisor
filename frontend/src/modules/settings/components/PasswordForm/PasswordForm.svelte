<script lang="ts">
  import type { UiContainer, UiNodeInputAttributes } from '@ory/kratos-client';
  import Button from 'components/Button/Button.svelte';
  import PasswordField from 'modules/forms/components/PasswordField/PasswordField.svelte';
  import PasswordToggle from 'modules/forms/components/PasswordToggle/PasswordToggle.svelte';
  import { Hint, required, useForm } from 'svelte-use-form';

  export let ui: UiContainer;
  const form = useForm();

  let type: 'password' | 'text' = 'password';

  const handleToggle = (e) => {
    e.preventDefault();
    const newType = type === 'password' ? 'text' : 'password';
    type = newType;
  };

  let fields = ui.nodes.reduce((acc, node) => {
    const { name, value } = node.attributes as UiNodeInputAttributes;
    acc[name] = {
      value: value || '',
    };
    return acc;
  }, {});
</script>

<form
  action={ui.action}
  method={ui.method}
  enctype="application/x-www-form-urlencoded"
  use:form
>
  <ul class="u-list-reset">
    <li class="visually-hidden">
      <input
        bind:value={fields['csrf_token'].value}
        type="hidden"
        name="csrf_token"
      />
    </li>
    <li>
      <PasswordField
        field={$form?.['password']}
        name={'password'}
        validate={required}
        placeholder={'Enter a new password'}
        {type}
      >
        <PasswordToggle
          on:click={handleToggle}
          activeType={type}
          slot="utilRight"
        />
        <svelte:fragment slot="label">Password</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </PasswordField>
    </li>
    <li>
      <Button
        style="secondary"
        size="medium"
        type="submit"
        name="method"
        value="password"
      >
        Save
      </Button>
    </li>
  </ul>
</form>
