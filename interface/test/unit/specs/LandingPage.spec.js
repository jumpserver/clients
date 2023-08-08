import Vue from 'vue'
import LandingPage from '@/layouts/LandingPage'

describe('LandingPage.vue', () => {
  it('should renderer correct contents', () => {
    const vm = new Vue({
      el: document.createElement('div'),
      render: h => h(LandingPage)
    }).$mount()

    expect(vm.$el.querySelector('.title').textContent).to.contain('Welcome to your new project!')
  })
})
