<template>
  <div class="register">
    <ul>
      <li><input v-model="account" placeholder="Account"/></li>
      <li><input v-model="name" placeholder="Name"/></li>
      <li><input type="password" v-model="password" placeholder="********"/></li>
      <li><button v-on:click="register">Register</button></li>
    </ul>
  </div>
</template>

<script>
import axios from 'axios'
import router from '@/router/index'
export default {
  name: 'register',
  data () {
    return {
      account: '',
      name: '',
      password: ''
    }
  },
  methods: {
    register () {
      axios.post('register/', {account: this.account, name: this.name, password: this.password}).then(res => {
        console.debug(this.$route.query.redirect)
        router.push('/')
      }).catch(err => {
        console.error(err)
      })
    }
  }
}

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
