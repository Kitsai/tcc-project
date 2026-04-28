export const useErrorToast = () => {
  const toast = useToast();
  const throwError = (text: string) => {
    toast.add({
      title: "Something went wrong!",
      description: text,
      color: "error",
    })
  }
  return throwError
}
