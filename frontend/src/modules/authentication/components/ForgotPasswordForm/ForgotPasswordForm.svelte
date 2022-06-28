<script lang="ts">
  import axios from 'axios';
  import Button from 'components/Button/Button.svelte';
  import { toast } from 'components/Toast/Toast';
  import { ENDPOINTS } from 'consts/endpoints';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { email, Hint, required, useForm } from 'svelte-use-form';

  const form = useForm();

  let activeType: 'password' | 'text' = 'password';

  const handleToggle = () => {
    activeType = activeType === 'password' ? 'text' : 'password';
  };

  async function submitPasswordReset() {
    const res = await axios.post(ENDPOINTS.AUTHENTICATION.FORGOT_PASSWORD, {
      email: $form.email.value,
    });

    if (res.status === 200) {
      toast.success(res.data);
    } else {
      toast.warning('An error occured');
    }
  }
</script>

<form
  use:form
  on:submit={(e) => {
    submitPasswordReset();
    e.preventDefault();
  }}
>
  <ul class="u-list-reset">
    <li class="s-bottom--medium">
      <Input
        size="medium"
        placeholder="Your e-mail"
        validate={[required, email]}
        name="email"
        field={$form?.email}
      >
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint on="email" hideWhenRequired>Email is not valid</Hint>
        </svelte:fragment>
      </Input>
    </li>
  </ul>

  <Button size="medium" display="block" style="primary" type="submit"
    >Reset Password</Button
  >
</form>
