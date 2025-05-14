# Council

# Table of contents

1. [Introduction](#introduction)
2. [How to use](#how-to-use)
   1. [Check that the origin is the council](#check-that-the-origin-is-the-council)
   2. [Create a proposal](#create-a-proposal)
   3. [Vote a proposal](#vote-a-proposal)
   4. [Execute the proposal](#execute-the-proposal)
   5. [Flow example](#flow-example)
3. [Use root](#use-root)

## Introduction

The council is a way to distribute the responsibility for some actions among multiple accounts. For example, to add a new validator to the network. If we did not have the council, a single private key would have that responsibility, being likely to be lost at some point, or be misused intentionally by its knowers. For this reason, a council mechanism is highly recommended.

In Stability, we use the [pallet collective](https://paritytech.github.io/substrate/master/pallet_collective/index.html) of substrate to do this mechanism.

## How to use

### Check that the origin is the council

In the places where we want the council to have control, we will have to provide a struct that implements the `EnsureOrigin` trait:

    - EnsureMember (Check that the origin is a member of the council).
    - EnsureMembers<n> (Checks that the origin is the council through a proposal with approved by at least n members)
    - EnsureProportionMoreThan<n, d> (Check that the origin is the council through a proposal with approved by more than n/d parts of the council members)
    - EnsureProportionAtLeast<n, d> (Check that the origin is the council through a proposal with approval by at least n/d parts of the council members)

An example of how to define would be:

```rs
pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 1, 2>;
```

Here we are defining a struct that implements the `EnsureOrigin` trait and that needs that at least 50% (1/2) of the council members have approved the proposal.

### Create a proposal

**1. Click "Extrinsics"**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/f9c41f55-32bd-4988-b754-1104730aff1d/ascreenshot.jpeg?tl_px=343,0&br_px=1836,840&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=262,56)

**2. Click here.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/7b498902-6060-40b0-93fb-745fbb885c81/ascreenshot.jpeg?tl_px=0,14&br_px=1493,854&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=139,139)

**3. Select a council. In our example the TechCommittee**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/eccd59ff-4835-4ce8-aa1a-4ce8c77ee66f/ascreenshot.jpeg?tl_px=0,418&br_px=1493,1258&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=123,139)

**4. Click here.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/ba239d6e-a2b3-4236-8e0f-a30ed56cf12d/ascreenshot.jpeg?tl_px=269,32&br_px=1762,872&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=262,139)

**5. Click here.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/a5eb7c07-5003-47e2-96bf-1e1cdb35af30/ascreenshot.jpeg?tl_px=227,304&br_px=1720,1144&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=262,139)

**6. Set the threshold which is the number of voters at which the proposal can be closed.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/9feb1a9d-7be2-4018-91ae-738741d15e97/ascreenshot.jpeg?tl_px=0,144&br_px=1493,984&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=210,139)

**7. Select the extrinsic you want to call from this proposal. In this example we will call the dispatchAsRoot of the rootController pallet.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/3e0767a7-8792-4426-b461-71585f47a97a/ascreenshot.jpeg?tl_px=0,500&br_px=1493,1340&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=174,139)

**8. Set lenghtbound. The lenghtbound is a parameter used to control the amount of gas used. The amount of gas does not depend on the lenghtbound, it depends on the byte size of the call, but to limit the amount of gas the lenghtbound parameter must be greater than the number of bytes in the call.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/ff994f0e-0712-458d-aeab-f04388377e08/ascreenshot.jpeg?tl_px=0,522&br_px=1493,1362&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=228,195)

**9. Sign and submit the extrinsic**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/334f2210-b80e-4a6f-b64e-56d5874d6386/ascreenshot.jpeg?tl_px=1386,382&br_px=2879,1222&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=309,139)

### Vote a proposal

**1. To vote on a proposal. Go to the blockchain events and look for the `Proposed` event of your council instance. You will need to copy the proposalHash and the index.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/28c28048-3970-419d-b861-0083d356463e/ascreenshot.jpeg?tl_px=1386,454&br_px=2879,1294&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=488,139)

**2. Click "Extrinsics"**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/936556db-568f-4550-b603-ec40daaefbac/ascreenshot.jpeg?tl_px=323,0&br_px=1816,840&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=262,58)

**3. Click here.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/e8f5d6ee-495f-4797-973f-902b479714f2/ascreenshot.jpeg?tl_px=0,32&br_px=1493,872&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=188,139)

**4. Select a council. In our example the "TechCommittee"**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/5a21b2b1-f305-49da-be17-4439225aa83f/ascreenshot.jpeg?tl_px=0,400&br_px=1493,1240&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=138,139)

**5. Click here.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/e07309cf-6020-403c-af75-fd966596b95f/ascreenshot.jpeg?tl_px=279,34&br_px=1772,874&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=262,139)

**6. Choose the extrinsic "vote"**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/61fe9ca8-0a38-4ab2-bd83-5575ada3727f/ascreenshot.jpeg?tl_px=235,452&br_px=1728,1292&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=262,139)

**7. Paste the "proposalHash" of the proposal you want to vote on.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/a363b191-b048-450a-a457-1a6544a0d7ff/ascreenshot.jpeg?tl_px=0,142&br_px=1493,982&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=193,139)

**8. Paste the "index" of the proposal you want to vote on.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/c6b73ffc-7208-4a4c-b74b-907dc5c0ea34/ascreenshot.jpeg?tl_px=0,264&br_px=1493,1104&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=163,139)

**9. Indicate whether you want to vote for or against**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/0d2fbb8c-dff5-446f-baa3-1e48f7122d95/ascreenshot.jpeg?tl_px=0,376&br_px=1493,1216&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=146,139)

**10. Sign and submit the extrinsic**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/26cfec15-11cd-4ec8-a6d5-3327caa370d8/ascreenshot.jpeg?tl_px=1386,636&br_px=2879,1476&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=409,198)

### Execute the proposal

**1. Click "Extrinsics"**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/bcdab124-4137-44ae-9ed6-4f8a6c2dc69d/ascreenshot.jpeg?tl_px=365,0&br_px=1858,840&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=262,52)

**2. Click here.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/b70dc66b-a299-4445-8516-ec9432f413e0/ascreenshot.jpeg?tl_px=0,402&br_px=1493,1242&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=168,139)

**3. Click here.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/c704f4a0-9174-450a-be3f-b645d947d8e9/ascreenshot.jpeg?tl_px=261,52&br_px=1754,892&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=262,139)

**4. Paste the proposal hash**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/504d8877-d296-4c4c-affc-5cd696835e63/ascreenshot.jpeg?tl_px=99,152&br_px=1592,992&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=262,139)

**5. Set the index of the proposal**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/ae1d4671-a51d-4d97-a917-1af8e3ab9261/ascreenshot.jpeg?tl_px=0,254&br_px=1493,1094&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=257,139)

**6. Set the `proposalWeighBound` which is the maximum weight to be spent by the dispatch linked to the proposal.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/be302119-dcc0-402a-bf13-18fde875b128/ascreenshot.jpeg?tl_px=0,402&br_px=1493,1242&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=159,139)

**7. Set lenghtBound. The lenghtBound is a parameter used to control the amount of gas used. The amount of gas does not depend on the lenghtBound, it depends on the byte size of the call, but to limit the amount of gas the lenghtbound parameter must be greater than the number of bytes in the call.**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/89cfbec9-5fd3-491d-a207-3ef5fa51d19e/ascreenshot.jpeg?tl_px=0,528&br_px=1493,1368&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=118,139)

**8. Click "Submit Transaction"**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/2417e193-1884-423d-9f41-9529db55efc3/ascreenshot.jpeg?tl_px=1386,636&br_px=2879,1476&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=461,264)

**9. Click "Sign and Submit"**

![](https://ajeuwbhvhr.cloudimg.io/colony-recorder.s3.amazonaws.com/files/2023-01-31/cd8a8f79-2090-4c44-8623-81605c38a88e/ascreenshot.jpeg?tl_px=1386,398&br_px=2879,1238&sharp=0.8&width=560&wat_scale=50&wat=1&wat_opacity=0.7&wat_gravity=northwest&wat_url=https://colony-labs-public.s3.us-east-2.amazonaws.com/images/watermarks/watermark_default.png&wat_pad=363,139)

### Flow example

```mermaid
sequenceDiagram
    participant Member1
    participant Member2
    participant Member3
    participant Member4
    participant Council
    participant ExamplePallet
    Member1->>Council: Create proposal
    Member1->>Council: Vote Yes (Extrinsic: vote)
    Member2->>Council: Vote Yes (Extrinsic: vote)
    Member3->>Council: Vote No (Extrinsic: vote)
    Member4->>+Council: Execute the proposal (Extrinsic: close)
    Council->>+ExamplePallet: (Extrinsic: example)
    ExamplePallet->>-Council:  Ok
    Council->>-Member4: Ok

```

## Use root

To be able to use parts of the code with root from the council in Stability we have created the RootController pallet, which allows to call from the `dispatch_as_root` extrinsic to another extrinsic with root as long as the origin of the call to `dispatch_as_root` is from a TechCommitteeInstance council proposal with at least 50% of the members approving this proposal as you can see here:

`runtime/src/lib.rs`

```rs
impl pallet_root_controller::Config for Runtime {
    type ControlOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 1, 2>;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
}
```

For example, to call the `forceTransfer` extrinsic, which is an extrinsic that requires the origin to be root, we must follow the following flow:

```mermaid
sequenceDiagram
    participant Member1
    participant Member2
    participant Member3
    participant Member4
    participant Council
    participant RootController
    participant Balances
    Member1->>Council: Create proposal (Extrinsic: propose) (proposal: balances::force_transfer)
    Member1->>Council: Vote Yes (Extrinsic: vote)
    Member2->>Council: Vote Yes (Extrinsic: vote)
    Member3->>Council: Vote No (Extrinsic: vote)
    Member4->>+Council: Execute the proposal (Extrinsic: close)
    Council->>+RootController: (Extrinsic: dispatch_as_root)
    RootController->>Balances: (Extrinsic: forceTransfer, Origin: Root)
    RootController->>-Council: OK
    Council->>-Member4: Ok
```
