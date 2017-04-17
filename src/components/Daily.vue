<template>
  <div class="daily">
    <h1>{{date}}</h1>
    <div class="member" v-for='entry in entries'>
      <h2>{{entry.member}}</h2>
      <h3>昨日したこと</h3>
      <ul id='yesterday'>
        <template v-if="entry.done.length > 0">
          <li v-for='work in entry.done'>{{work.title}}</li>
        </template>
        <li v-else>なし</li>
      </ul>
      <h3>今日すること</h3>
      <ul id='today'>
        <template v-if="entry.to_do.length > 0">
          <li v-for='work in entry.to_do'>{{work.title}}</li>
        </template>
        <li v-else>なし</li>
      </ul>
      <h3>障害/業務外の予定</h3>
      <ul id='problem'>
        <template v-if="entry.problem.length > 0">
          <li v-for='issue in entry.problem'>{{issue.title}}</li>
        </template>
        <li v-else>なし</li>
      </ul>
    </div>
  </div>
</template>

<script>
import axios from 'axios'
export default {
  name: 'daily',
  data () {
    return {
      date: (new Date()).toLocaleDateString(),
      entries: []
    }
  },
  mounted () {
    axios.get('data/').then(res => {
      this.date = res.data.date
      this.entries = res.data.entries
    }).catch(err => {
      console.error(err)
    })
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
div.member {
  display: inline-block;
}
</style>
