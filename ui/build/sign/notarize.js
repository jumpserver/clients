const { notarize } = require('@electron/notarize');

const {
  XCODE_APP_LOADER_EMAIL, // 你的apple Id
  XCODE_APP_LOADER_PASSWORD, // 准备工作中生成的app专用密码，注意不能用apple Id的密码
  XCODE_APP_TEAM_ID // Team ID
} = process.env;

async function main(context) {
  const { electronPlatformName, appOutDir } = context;

  if (
    electronPlatformName !== 'darwin' ||
    !XCODE_APP_LOADER_EMAIL ||
    !XCODE_APP_LOADER_PASSWORD ||
    !XCODE_APP_TEAM_ID
  ) {
    console.log('Skipping Apple notarization.');
    return;
  }

  console.log('Starting Apple notarization.');
  const appName = context.packager.appInfo.productFilename;

  await notarize({
    appBundleId: 'com.jumpserver.client', // 将com.example.app替换为你自己项目的id
    appPath: `${appOutDir}/${appName}.app`,
    teamId: XCODE_APP_TEAM_ID,
    appleId: XCODE_APP_LOADER_EMAIL,
    appleIdPassword: XCODE_APP_LOADER_PASSWORD
  });

  console.log('Finished Apple notarization.');
}

exports.default = main;
