# 로드맵

> [`04-fork-plan.md`](./04-fork-plan.md)의 착수 계획 + [`02-oss-survey.md`](./02-oss-survey.md)의 채택 순서 +
> [`03-harness-design.md`](./03-harness-design.md)의 원칙/벤치마크를 하나의 버전 시퀀스로 합친 것. 각 항목은
> 위 세 문서 중 하나를 가리키며, 여기서 로직을 다시 설명하지 않는다.

## v0.1 — Bring-up & 안전 기반 (아무 독창 기능보다 먼저)

기능 추가보다 "사고 안 나는 기반"을 먼저 깐다 — 특히 샌드박싱은 협상 불가.

- [`04-fork-plan.md`](./04-fork-plan.md) 0~2단계: 라이선스/조직 재확인 → fork → provider json만으로 core
  무수정 라우팅 검증
- **[필수, 최우선]** OS 커널 샌드박스 추가([`04-fork-plan.md`](./04-fork-plan.md) "추가 확정 사항",
  [`03-harness-design.md`](./03-harness-design.md) 원칙 11) — Goose는 자체 샌드박스가 없다는 게 확인된 가장
  중요한 갭
- [`02-oss-survey.md`](./02-oss-survey.md) 채택순서 1~6: PLEUM.md 메모리 컨벤션, 권한 정책 파일, shadow-git
  체크포인트, 편집 전 린트 게이트, 스테이지드-diff-only 기본값, 세션 자동 저장/재개

## v0.2 — 코딩 신뢰성 기반 (ACI + 검증 루프)

- [`03-harness-design.md`](./03-harness-design.md) 원칙 1: ground-truth 검증 루프(`verify_cmd`, 3회 캡)
- [`03-harness-design.md`](./03-harness-design.md) 원칙 7: ACI 설계(windowed view, 좁은 navigation, 편집
  가드레일 — v0.1의 린트게이트와 통합)
- [`02-oss-survey.md`](./02-oss-survey.md) 채택순서 7~9: tree-sitter repo-map, AST 기반 구조 검색 tool
  (클린룸 구현), pluggable @-context-provider
- [`03-harness-design.md`](./03-harness-design.md) 원칙 9: 결정론적 체크 우선 → 추론적 체크 후순위 순서 확립

## v0.3 — 멀티모델 서브에이전트 오케스트레이션 엔진 (핵심 독창 기능)

[`01-unique-features.md`](./01-unique-features.md)의 최우선 확정 기능. 여기서 만드는 "역할→모델 배정
디스패치"가 v0.4의 모든 프리셋의 기반이 된다.

- Goose native subagent/subrecipe에 role별 `settings.goose_model` 배정 ([`01-unique-features.md`](./01-unique-features.md)
  "핵심 통찰" 참고, 업스트림 버그 우회 필수)
- [`03-harness-design.md`](./03-harness-design.md) "핵심 설계 원칙": 오케스트레이터-워커 공통 계약(`role`,
  `model`, `context_scope` → `{result, changed_files, verdict?, confidence?}`) 확정
- [`02-oss-survey.md`](./02-oss-survey.md) 채택순서 14: Boomerang식 오케스트레이터→컨텍스트 격리 서브태스크
  위임
- [`03-harness-design.md`](./03-harness-design.md) 원칙 2, 3: 리스크 티어링 best-of-N + execute-and-vote,
  rollout 구조화 요약 압축
- [`03-harness-design.md`](./03-harness-design.md) 원칙 5: adversarial review (frozen-artifact Reviewer,
  evidence-typed verdict)

## v0.4 — 프리셋 계층 (Cascade / Fusion / Smart Mode / Plan-Act)

v0.3 엔진 위에 얹는 UX 프리셋. 새 메커니즘을 만들지 않고 v0.3의 디스패치를 재사용한다.

- Cascade 검증 하네스 = 순차 프리셋([`01-unique-features.md`](./01-unique-features.md))
- Fusion 병렬 구현/리뷰 = 병렬 프리셋 + routing-policy 피커 슬래시 명령(`/policy`, `/smart`,
  [`04-fork-plan.md`](./04-fork-plan.md) 5~6단계)
- Smart Mode = 자동 프리셋(서버사이드 패스스루, 거의 공짜)
- [`02-oss-survey.md`](./02-oss-survey.md) 채택순서 10~11: Architect/Editor 2-모델 분리 + Fast-Apply,
  Plan/Act 모드 명시적 분리
- [`03-harness-design.md`](./03-harness-design.md) 원칙 6: 신뢰도×가역성 2축 에스컬레이션(`GOOSE_MODE`
  확장)
- [`03-harness-design.md`](./03-harness-design.md) 원칙 8: 조기 완료선언 가로채기 + progress.txt 세션 복구

## v0.5 — KRW 비용/예산 + 멀티모달 tool

- KRW 실시간 비용 한 줄 출력 + 예산 가드레일([`04-fork-plan.md`](./04-fork-plan.md) 4단계, 서브에이전트별
  비용 분해는 [`01-unique-features.md`](./01-unique-features.md) #5)
- 멀티모달 tool 통합(이미지/음성/영상 생성, [`01-unique-features.md`](./01-unique-features.md) #2)
- [`02-oss-survey.md`](./02-oss-survey.md) 채택순서 12~13: 자가검증 루프(테스트먼저생성→실행→수정), 커밋
  메시지 자동생성 서브커맨드

## v1.0-beta — Opt-in 베타 출시

- [`04-fork-plan.md`](./04-fork-plan.md) 7~8단계: 업스트림 리베이스 주기 확정, `pleum-code` 별도 바이너리로
  opt-in 베타 배포(기존 `pleum` 10-레시피 런처는 그대로 병행)
- GitHub Public 저장소 공개 ([`00-vision.md`](./00-vision.md) 확정: 저장소명 `pleumcode`, PleumRouter와
  같은 조직)
- [`03-harness-design.md`](./03-harness-design.md) 벤치마크 목표 가동: 내부 dogfooding 스위트(최우선) +
  SWE-bench Pro/Multimodal v2/Terminal-Bench 4.0을 `goose bench`에 어댑터로 추가, SWE-bench Verified는
  회귀 sanity check로만
- 릴리스 전 정기 레드팀(AIShellJack류 ASR 측정) 프로세스 가동 ([`03-harness-design.md`](./03-harness-design.md)
  안전성 권고)

## v1.0 이후 후보 (스코프 아웃)

- 한국어 네이티브 UX + PIPA 가드레일([`01-unique-features.md`](./01-unique-features.md) "v2 이후 후보")
- IDE 확장([`00-vision.md`](./00-vision.md)에서 v1 범위 밖으로 확정 — 로드맵으로만 기록)
- [`02-oss-survey.md`](./02-oss-survey.md) 채택순서 15~24: OS 샌드박스(→ v0.1로 이미 앞당김), Docker 원격
  실행 확장, 이벤트-스트림 아키텍처 전환, CodeAct식 액션 통일, Zed식 멀티 워크트리 동시실행,
  Everything-is-a-plugin 통합 리팩터링, ACP 외부 프로토콜 노출, zero-install 웹 온보딩 데모
