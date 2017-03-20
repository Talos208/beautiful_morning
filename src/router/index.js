import Vue from 'vue'
import Router from 'vue-router'
import Daily from '@/components/Daily'

Vue.use(Router)

export default new Router({
  routes: [
    {
      path: '/',
      name: 'Daily',
      component: Daily
    }
  ]
})
