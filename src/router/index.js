import Vue from 'vue'
import Router from 'vue-router'
import Daily from '@/components/Daily'
import Login from '@/components/Login'

Vue.use(Router)

var router = new Router({
  routes: [
    {
      path: '/',
      name: 'Daily',
      component: Daily,
      meta: {requireAuth: true}
    },
    {
      path: '/login',
      name: 'Login',
      component: Login
    }
  ]
})

router.beforeEach((to, from, next) => {
  if (to.matched.some(record => record.meta.requireAuth) && !Login.loggedIn) {
    next({path: '/login', query: {redirect: to.fullPath}})
  } else {
    next()
  }
})

export default router

