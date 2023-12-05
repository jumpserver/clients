<template>
  <div>
    <img id="logo" src="@/assets/JumpServer.png" alt=""/>
    <p>{{ $t('Common.JumpServerClient') }}</p>
    <p>{{ $t('AboutUs.Version') }}：v2.1.0</p>
    <p>{{ $t('AboutUs.Copyright') }}：FIT2CLOUD 飞致云 © 2014-2023 {{ $t('AboutUs.AllRightsReserved')}}</p>
    <a target="_blank" href="https://www.fit2cloud.com/jumpserver/index.html">{{ $t('AboutUs.OfficialWebsite') }}</a>
    <a target="_blank" href="https://docs.jumpserver.org/zh/v3/">{{ $t('AboutUs.OnlineDocumentation') }}</a>
    <a target="_blank" href="https://kb.fit2cloud.com/categories/jumpserver">{{ $t('AboutUs.KnowledgeBase') }}</a>
    <a target="_blank" href="https://www.fit2cloud.com/about/index.html">{{ $t('AboutUs.AboutUs') }}</a>
  </div>
</template>

<script>
export default {
  name: "AboutPage",
  components: {},
  setup() {}
}
</script>

<style lang="scss" scoped>
div {
  width: 100%;
  height: 100%;
  text-align: center;
  color: #e1e1e1e1;

  p {
    font-size: 15px;
  }

  a {
    line-height: 50px;
    margin: 0 15px;
    color: #3f83cc;
  }

  #logo {
    margin-top: 80px;
    width: 200px;
  }
}
</style>
