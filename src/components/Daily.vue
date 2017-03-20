<template>
  <div class="daily">
    <h1>{{date}}</h1>
    <div v-for='person in members'>
      <h2>{{person.name}}</h2>
      <h3>昨日したこと</h3>
      <ul id='yesterday'>
        <li v-for='work in person.yesterday'>{{work.title}}</li>
      </ul>
      <h3>今日すること</h3>
      <ul id='today'>
      <li v-for='work in person.today'>{{work.title}}</li>
      </ul>
      <h3>障害/業務外の予定</h3>
      <ul id='problem'>
      <li v-for='issue in person.problem'>{{issue.title}}</li>
      </ul>
    </div>
  </div>
</template>

<script>
export default {
  name: 'daily',
  data () {
    return {
      date: (new Date()).toLocaleDateString(),
      members: [{
        name: '太郎',
        yesterday: [{
          title: 'チケット#2'
        },
        {
          title: 'チケット#4'
        }],
        today: [{title: 'チケット#3'}],
        problem: [{title: '結合テスト環境が動いていない'}]
      }]
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
