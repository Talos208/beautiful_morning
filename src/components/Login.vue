<template>
  <div class="login">
    <input v-model="account" placeholder="Account"/>
    <input type="password" v-model="password" placeholder="********"/>
    <button v-on:click="login">Login</button>
    <router-link to="register">Register...</router-link>
  </div>
</template>

<script>
import axios from 'axios'
import router from '@/router/index'
var login = {
  name: 'login',
  loggedIn: false,
  authToken: null,
  data () {
    return {
      account: '',
      password: ''
    }
  },
  methods: {
    login () {
      var account = this.account
      var password = this.password
      axios.post('login/', {account: account, password: password}).then(res => {
        login.loggedIn = res.data.success
        login.authToken = res.data.token
        router.push(this.$route.query.redirect)
      }).catch(err => {
        console.error(err)
      })
    }
  }
}

export default login

</script>

<style scoped>
h3 {
  margin: 0 .5em;
}
ul {
  list-style-type: none;
  padding: 0;
}

li {
  display: inline-block;
  margin: 0 1em;
}
</style>
