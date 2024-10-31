import type {
  RouteLocationNormalized,
  RouteLocationNormalizedLoaded,
  NavigationGuardNext
} from 'vue-router';

export const guard = (
  to: RouteLocationNormalized,
  from: RouteLocationNormalizedLoaded,
  next: NavigationGuardNext
) => {
  // console.log(to);
  // console.log(from);
  // console.log(next);

  // window.location = 'http://192.168.200.8:8080/core/auth/login/';

  next();
};
